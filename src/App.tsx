import "./App.css";

function App() {
  return (
    <main className="app-shell">
      <header className="app-header">
        <div>
          <p className="eyebrow">MYTORY YT-DLP</p>
          <h1>Download Queue</h1>
        </div>
        <button type="button" disabled title="Download Queue 구현 후 사용할 수 있습니다.">
          URL 추가
        </button>
      </header>

      <section className="queue-card" aria-labelledby="queue-heading">
        <div className="queue-empty" role="status">
          <h2 id="queue-heading">대기열이 비어 있습니다</h2>
          <p>
            URL 추가, 저장 위치 선택, 다운로드 재개 기능은 다음 수직 슬라이스에서
            연결됩니다.
          </p>
        </div>
      </section>
    </main>
  );
}

export default App;
