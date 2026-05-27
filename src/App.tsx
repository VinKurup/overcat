import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

interface Cat {
  name: string;
  personality: string;
  satiety: number;
  happiness: number;
  energy: number;
  mood: string;
}

const moodEmoji: Record<string, string> = {
  Idle: "😺",
  Sad: "😿",
  Sick: "🙀",
  Sleepy: "😴",
};

function StatBar({ label, value }: { label: string; value: number }) {
  return (
    <div className="stat">
      <span className="stat-label">{label}</span>
      <div className="stat-track">
        <div className="stat-fill" style={{ width: `${value}%` }} />
      </div>
      <span className="stat-value">{value}</span>
    </div>
  );
}

function App() {
  const [cat, setCat] = useState<Cat | null>(null);

  useEffect(() => {
    invoke<Cat>("get_cat_state").then(setCat).catch(console.error);
  }, []);

  if (!cat) return <main className="container">Loading…</main>;

  return (
    <main className="container">
      <h1>
        {moodEmoji[cat.mood] ?? "🐱"} {cat.name}
      </h1>
      <p className="subtitle">
        {cat.personality} · {cat.mood}
      </p>

      <div className="stats">
        <StatBar label="Satiety" value={cat.satiety} />
        <StatBar label="Happiness" value={cat.happiness} />
        <StatBar label="Energy" value={cat.energy} />
      </div>

      <div className="row">
        <button onClick={() => invoke<Cat>("feed_cat").then(setCat)}>Feed</button>
        <button onClick={() => invoke<Cat>("play_with_cat").then(setCat)}>Play</button>
      </div>
    </main>
  );
}

export default App;
