import { invoke } from "@tauri-apps/api/core";

export interface Note {
    id: number;
    text: string;
    priority: number;
    done: boolean;
}

export interface ListNotesArgs {
    done?: boolean;
    todo?: boolean;
    contains?: string;
    priority?: number;
}

export const addNote = (text: string, priority = 0) =>
    invoke<number>("add_note", { text, priority });

export const listNotes = (args: ListNotesArgs = {}) => {
    const done = args.done ?? false;
    const todo = args.todo ?? false;
    const contains = args.contains ?? null;
    const priority = args.priority ?? null;

    return invoke<Note[]>("list_notes", {
        done,
        todo,
        contains,
        priority,
    });
};
