import { invoke } from "@tauri-apps/api/core";
import { FormEvent, useEffect, useState } from "react";
import "./App.css";

type DownloadStatus = "queued" | "running" | "completed" | "failed" | "cancelled";
type QueueJob = { id: string; source_url: string; destination: string; status: DownloadStatus };

function App() {
  const [urls, setUrls] = useState("");
  const [destination, setDestination] = useState("");
  const [jobs, setJobs] = useState<QueueJob[]>([]);
  const [concurrency, setConcurrency] = useState(3);
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
    if (!entries.length || !destination.trim()) {
      setError("URL과 저장 위치를 입력하세요.");
      return;
    }
    setSubmitting(true);
    setError("");
    try {
      await invoke("enqueue_downloads", { request: { urls: entries, destination: destination.trim() } });
      setUrls("");
      await refresh();
    } catch (reason) {
      setError(String(reason));
    } finally {
      setSubmitting(false);
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

  return <main className="app-shell">
    <header className="app-header"><div><p className="eyebrow">MYTORY YT-DLP</p><h1>Download Queue</h1></div></header>
    <form className="add-form" onSubmit={submit}>
      <label>URL <textarea value={urls} onChange={(event) => setUrls(event.target.value)} placeholder="한 줄에 URL 하나씩 입력하세요" required /></label>
      <label>저장 위치 <input value={destination} onChange={(event) => setDestination(event.target.value)} required /></label>
      <label>동시 다운로드 <select value={concurrency} onChange={(event) => void updateConcurrency(Number(event.target.value))}>{[1,2,3,4,5].map((value) => <option key={value} value={value}>{value}</option>)}</select></label>
      <button type="submit" disabled={submitting}>{submitting ? "추가 중…" : "URL 추가"}</button>
    </form>
    {error && <p className="error" role="alert">{error}</p>}
    <section className="queue-card" aria-labelledby="queue-heading"><h2 id="queue-heading">작업 목록</h2>{jobs.length ? <ul>{jobs.map((job) => <li key={job.id}><strong>{job.status}</strong><span>{job.source_url}</span><small>{job.destination}</small></li>)}</ul> : <p>대기열이 비어 있습니다.</p>}</section>
  </main>;
}
export default App;
