import { useEffect, useState } from "react";
import { addNote, listNotes } from "./lib/noteApi";
import type { Note } from "./lib/noteApi";

type DraftNote = {
    text: string;
    priority: number;
};

export default function App() {
    const [draft, setDraft] = useState<DraftNote>({
        text: "",
        priority: 0,
    });
    const [notes, setNotes] = useState<Note[]>([]);
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState("");

    const refresh = async () => {
        setLoading(true);
        setError("");
        try {
            const data = await listNotes();
            setNotes(data);
        } catch (error) {
            setError(String(error));
        } finally {
            setLoading(false);
        }
    };

    const onAdd = async () => {
        const text = draft.text.trim();
        if (!text) return;

        setError("");
        try {
            await addNote(text, draft.priority);
            setDraft({ text: "", priority: 0 });
            await refresh();
        } catch (error) {
            setError(String(error));
        }
    };

    useEffect(() => {
        void refresh();
    }, []);

    return (
        <main className="container">
            <h1>Novel Notes</h1>

            <div className="row">
                <input
                    value={draft.text}
                    onChange={(e) => {
                        const value = e.currentTarget.value;
                        setDraft((prev) => ({
                            ...prev,
                            text: value,
                        }));
                    }}
                />
                <input
                    type="number"
                    min={0}
                    max={5}
                    value={draft.priority}
                    onChange={(e) => {
                        const raw = e.currentTarget.value;
                        const next = raw === "" ? 0 : Number(raw);
                        setDraft((prev) => ({
                            ...prev,
                            priority: Number.isFinite(next) ? next : 0,
                        }));
                    }}
                />
                <button onClick={onAdd} disabled={loading}>
                    Add
                </button>
                <button onClick={refresh} disabled={loading}>
                    Refresh
                </button>
            </div>

            {error && <p style={{ color: "crimson" }}>{error}</p>}

            <ul>
                {notes.map((note) => (
                    <li key={note.id}>
                        [{note.done ? "x" : " "}] P{note.priority} #{note.id}{" "}
                        {note.text}
                    </li>
                ))}
            </ul>

            {!loading && notes.length === 0 && <p>No notes found.</p>}
        </main>
    );
}
