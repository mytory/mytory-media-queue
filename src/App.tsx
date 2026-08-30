import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { FormEvent, useEffect, useState } from "react";
import "./App.css";

type DownloadStatus = "queued" | "running" | "completed" | "failed" | "cancelled";
type OutputPreset = "mp4_compatible" | "best_video" | "original_audio" | "mp3_320";
type QueueJob = { id: string; source_url: string; destination: string; status: DownloadStatus };

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
  const [error, setError] = useState("");
  const [submitting, setSubmitting] = useState(false);

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
      await invoke("enqueue_downloads", { request: { urls: entries, destination: normalizedDestination, outputPreset } });
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

  async function updateConcurrency(value: number) {
    setConcurrency(value);
    try {
      await invoke("set_download_concurrency", { concurrency: value });
    } catch (reason) {
      setError(String(reason));
      await refresh();
    }
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
            <label><span>저장 위치</span><div className="destination-picker"><input value={destination} readOnly required /><button type="button" onClick={() => void chooseDestination()}>폴더 선택</button></div></label>
            <label><span>형식</span><select value={outputPreset} onChange={(event) => setOutputPreset(event.target.value as OutputPreset)}><option value="mp4_compatible">MP4 호환 우선</option><option value="best_video">최고 품질 영상</option><option value="original_audio">원본 품질 오디오</option><option value="mp3_320">MP3 320kbps</option></select></label>
            <label className="concurrency"><span>동시 작업</span><select value={concurrency} onChange={(event) => void updateConcurrency(Number(event.target.value))}>{[1, 2, 3, 4, 5].map((value) => <option key={value} value={value}>{value}개</option>)}</select></label>
          </div>
          <div className="form-footer"><p>기본 형식: MP4 호환 · 썸네일 저장</p><button type="submit" disabled={submitting}>{submitting ? "대기열에 넣는 중" : "대기열에 추가"}<span>→</span></button></div>
        </form>

        <section className="queue-card" aria-labelledby="queue-heading">
          <div className="queue-heading"><div><p className="eyebrow">QUEUE / {jobs.length.toString().padStart(2, "0")}</p><h2 id="queue-heading">작업 목록</h2></div><button className="quiet-button" type="button" onClick={() => void refresh()}>새로고침</button></div>
          {jobs.length ? <ol className="job-list">{jobs.map((job, index) => <li key={job.id}><span className="job-index">{String(index + 1).padStart(2, "0")}</span><div className="job-content"><strong>{job.source_url}</strong><small>{job.destination}</small></div><span className={`status status-${job.status}`}>{statusLabel[job.status]}</span></li>)}</ol> : <div className="empty-state"><span>↓</span><h3>아직 작업이 없습니다</h3><p>위에 URL을 입력하면 이곳에서 순서와 상태를 확인할 수 있습니다.</p></div>}
        </section>
      </section>
      {error && <p className="error" role="alert">{error}</p>}
    </main>
  );
}

export default App;
