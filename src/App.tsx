import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { FormEvent, useEffect, useRef, useState } from "react";
import "./App.css";

type DownloadStatus = "queued" | "running" | "completed" | "failed" | "cancelled";
type OutputPreset = "mp4_compatible" | "best_video" | "original_audio" | "mp3_320";
type FailureKind = "transient_network" | "permission" | "interrupted" | "unknown";
type QueueJob = {
  id: string;
  source_url: string;
  destination: string;
  output_preset: OutputPreset;
  write_subs: boolean;
  status: DownloadStatus;
  progress_percent: number | null;
  speed_bytes_per_second: number | null;
  eta_seconds: number | null;
  attempt_count: number;
  failure_kind: FailureKind | null;
  diagnostic_log: string | null;
};

const statusLabel: Record<DownloadStatus, string> = {
  queued: "대기 중",
  running: "진행 중",
  completed: "완료",
  failed: "확인 필요",
  cancelled: "취소됨",
};

function App() {
  const [urls, setUrls] = useState("");
  const [destination, setDestination] = useState("");
  const [jobs, setJobs] = useState<QueueJob[]>([]);
  const [concurrency, setConcurrency] = useState(3);
  const [outputPreset, setOutputPreset] = useState<OutputPreset>("mp4_compatible");
  const [writeSubs, setWriteSubs] = useState(false);
  const [cookies, setCookies] = useState("");
  const [error, setError] = useState("");
  const [submitting, setSubmitting] = useState(false);
  const [pending, setPending] = useState<Set<string>>(new Set());
  const [confirmClearOpen, setConfirmClearOpen] = useState(false);
  const cancelRef = useRef<HTMLButtonElement>(null);

  const refresh = async () => {
    try {
      setJobs(await invoke<QueueJob[]>("list_downloads"));
      setConcurrency(await invoke<number>("get_download_concurrency"));
    } catch (reason) {
      setError(String(reason));
    }
  };

  useEffect(() => {
    void Promise.all([invoke<string>("default_download_destination"), refresh()])
      .then(([path]) => setDestination(path))
      .catch((reason) => setError(String(reason)));
    const interval = window.setInterval(() => void refresh(), 1000);
    return () => window.clearInterval(interval);
  }, []);

  async function submit(event: FormEvent) {
    event.preventDefault();
    const entries = urls.split(/\r?\n/).map((url) => url.trim()).filter(Boolean);
    const normalizedDestination = destination.trim();
    if (!entries.length || !normalizedDestination) {
      setError("URL과 저장 위치를 입력하세요.");
      return;
    }
    setSubmitting(true);
    setError("");
    try {
      await invoke("enqueue_downloads", {
        request: {
          urls: entries,
          destination: normalizedDestination,
          outputPreset,
          writeSubs,
          cookies: cookies || null,
        },
      });
      setUrls("");
      await refresh();
    } catch (reason) {
      setError(String(reason));
    } finally {
      setSubmitting(false);
    }
  }

  async function chooseDestination() {
    try {
      const selected = await open({ directory: true, multiple: false, defaultPath: destination });
      if (typeof selected === "string") setDestination(selected);
    } catch (reason) {
      setError(String(reason));
    }
  }

  async function chooseCookies() {
    try {
      const selected = await open({ directory: false, multiple: false });
      if (typeof selected === "string") setCookies(selected);
    } catch (reason) {
      setError(String(reason));
    }
  }

  async function runAction(id: string, action: string, command: string) {
    const key = `${id}:${action}`;
    setPending((ids) => new Set(ids).add(key));
    setError("");
    try {
      await invoke(command, { id });
      await refresh();
    } catch (reason) {
      setError(String(reason));
    } finally {
      setPending((ids) => {
        const next = new Set(ids);
        next.delete(key);
        return next;
      });
    }
  }

  async function copyDiagnostic(text: string) {
    try {
      await navigator.clipboard.writeText(text);
    } catch {
      const area = document.createElement("textarea");
      area.value = text;
      document.body.appendChild(area);
      area.select();
      document.execCommand("copy");
      document.body.removeChild(area);
    }
  }

  async function confirmClearHistory() {
    const key = "history:clear";
    setPending((ids) => new Set(ids).add(key));
    setError("");
    try {
      await invoke("clear_history");
      setConfirmClearOpen(false);
      await refresh();
    } catch (reason) {
      setError(String(reason));
    } finally {
      setPending((ids) => {
        const next = new Set(ids);
        next.delete(key);
        return next;
      });
    }
  }

  useEffect(() => {
    if (!confirmClearOpen) return;
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") setConfirmClearOpen(false);
    };
    window.addEventListener("keydown", onKey);
    cancelRef.current?.focus();
    const previousOverflow = document.body.style.overflow;
    document.body.style.overflow = "hidden";
    return () => {
      window.removeEventListener("keydown", onKey);
      document.body.style.overflow = previousOverflow;
    };
  }, [confirmClearOpen]);

  async function updateConcurrency(value: number) {
    setConcurrency(value);
    try {
      await invoke("set_download_concurrency", { concurrency: value });
    } catch (reason) {
      setError(String(reason));
      await refresh();
    }
  }

  function actionButton(job: QueueJob, action: string, command: string, label: string, tone = "") {
    const key = `${job.id}:${action}`;
    return (
      <button
        key={key}
        className={`job-button${tone ? ` ${tone}` : ""}`}
        type="button"
        disabled={pending.has(key)}
        onClick={() => void runAction(job.id, action, command)}
      >
        {label}
      </button>
    );
  }

  function jobActions(job: QueueJob) {
    const buttons: React.ReactNode[] = [];
    if (job.status === "failed" || job.status === "cancelled") {
      buttons.push(actionButton(job, "retry", "retry_download", "재시도"));
    }
    if (job.status === "failed" && job.diagnostic_log) {
      buttons.push(
        <button key="copy" className="job-button" type="button" onClick={() => void copyDiagnostic(job.diagnostic_log!)}>
          로그 복사
        </button>,
      );
    }
    if (job.status === "completed" || job.status === "failed") {
      buttons.push(actionButton(job, "open", "open_download_folder", "폴더 열기"));
    }
    if (job.status === "queued" || job.status === "running") {
      buttons.push(actionButton(job, "cancel", "cancel_download", "취소", "danger"));
    }
    if (job.status !== "completed") {
      buttons.push(actionButton(job, "remove", "remove_download", "제거", "danger"));
    }
    return <div className="job-actions">{buttons}</div>;
  }

  const activeCount = jobs.filter((job) => job.status === "running").length;
  const queuedCount = jobs.filter((job) => job.status === "queued").length;

  return (
    <main className="app-shell">
      <header className="app-header">
        <div className="brand-lockup"><span className="brand-mark">↓</span><div><p className="eyebrow">MYTORY / DOWNLOAD DESK</p><h1>유튜브, SNS 영상 다운로더</h1></div></div>
        <div className="queue-meter" aria-label={`진행 ${activeCount}개, 대기 ${queuedCount}개`}><span className="meter-dot" /><span>진행 {activeCount}</span><span className="meter-divider">/</span><span>대기 {queuedCount}</span></div>
      </header>

      <section className="workspace" aria-label="다운로드 작업 영역">
        <form className="add-form" onSubmit={submit}>
          <div className="form-heading"><span>새 작업</span><p>주소를 한 줄에 하나씩 붙여 넣으세요.</p></div>
          <label className="url-field"><span>미디어 URL</span><textarea value={urls} onChange={(event) => setUrls(event.target.value)} placeholder={"https://…\nhttps://…"} required /></label>
          <div className="form-options">
            <label className="destination-field"><span>저장 위치</span><div className="destination-picker"><input value={destination} readOnly required /><button type="button" onClick={() => void chooseDestination()}>폴더 선택</button></div></label>
            <label className="subs-field"><span>부가 파일</span><label className="subs-toggle"><input type="checkbox" checked={writeSubs} onChange={(event) => setWriteSubs(event.target.checked)} /><span>자막 저장 · 한국어·영어 (.vtt)</span></label></label>
            <label className="cookie-field"><span>로그인 쿠키 (선택)</span><div className="cookie-picker"><button type="button" onClick={() => void chooseCookies()} title={cookies}>{cookies ? cookies.split(/[\\/]/).pop() : "cookies.txt 선택"}</button>{cookies ? <button type="button" onClick={() => setCookies("")}>해제</button> : null}</div></label>
            <label><span>형식</span><select value={outputPreset} onChange={(event) => setOutputPreset(event.target.value as OutputPreset)}><option value="mp4_compatible">MP4 호환 우선</option><option value="best_video">최고 품질 영상</option><option value="original_audio">원본 품질 오디오</option><option value="mp3_320">MP3 320kbps</option></select></label>
            <label className="concurrency"><span>동시 작업</span><select value={concurrency} onChange={(event) => void updateConcurrency(Number(event.target.value))}>{[1, 2, 3, 4, 5].map((value) => <option key={value} value={value}>{value}개</option>)}</select></label>
          </div>
          <div className="form-footer"><p>기본 형식: MP4 호환 · 썸네일 저장</p><button type="submit" disabled={submitting}>{submitting ? "대기열에 넣는 중" : "대기열에 추가"}<span>→</span></button></div>
        </form>

        <section className="queue-card" aria-labelledby="queue-heading">
          <div className="queue-heading"><div><p className="eyebrow">QUEUE / {jobs.length.toString().padStart(2, "0")}</p><h2 id="queue-heading">작업 목록</h2></div><div className="queue-heading-actions"><button className="quiet-button" type="button" onClick={() => setConfirmClearOpen(true)}>이력 지우기</button><button className="quiet-button" type="button" onClick={() => void refresh()}>새로고침</button></div></div>
          {jobs.length ? <ol className="job-list">{jobs.map((job, index) => <li key={job.id}><span className="job-index">{String(index + 1).padStart(2, "0")}</span><div className="job-content"><strong>{job.source_url}</strong><small>{job.destination}</small>{job.status === "failed" && job.diagnostic_log ? <details className="job-diagnostic"><summary>진단 로그{job.attempt_count > 0 ? ` · ${job.attempt_count}회 재시도 후` : ""}</summary><pre>{job.diagnostic_log}</pre></details> : null}</div><span className={`status status-${job.status}`}>{job.status === "running" && job.progress_percent !== null ? `${Math.round(job.progress_percent)}%` : statusLabel[job.status]}</span>{jobActions(job)}</li>)}</ol> : <div className="empty-state"><span>↓</span><h3>아직 작업이 없습니다</h3><p>위에 URL을 입력하면 이곳에서 순서와 상태를 확인할 수 있습니다.</p></div>}
        </section>
      </section>
      {confirmClearOpen && (
        <div className="modal-backdrop" onClick={() => setConfirmClearOpen(false)}>
          <div className="modal-card" role="dialog" aria-modal="true" aria-labelledby="clear-history-title" onClick={(event) => event.stopPropagation()}>
            <p className="eyebrow">HISTORY</p>
            <h3 id="clear-history-title" className="modal-title">이력 지우기</h3>
            <p className="modal-body">완료·실패·취소된 작업 기록이 목록에서 삭제됩니다. 내려받은 파일은 그대로 유지됩니다.</p>
            <div className="modal-actions">
              <button ref={cancelRef} className="modal-cancel" type="button" onClick={() => setConfirmClearOpen(false)}>취소</button>
              <button className="modal-confirm" type="button" disabled={pending.has("history:clear")} onClick={() => void confirmClearHistory()}>{pending.has("history:clear") ? "지우는 중" : "이력 지우기"}</button>
            </div>
          </div>
        </div>
      )}
      {error && <p className="error" role="alert">{error}</p>}
    </main>
  );
}

export default App;
