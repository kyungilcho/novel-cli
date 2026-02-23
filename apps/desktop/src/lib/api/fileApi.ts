import { invoke } from "@tauri-apps/api/core";

export type FileEntry = {
    path: string;
    is_dir: boolean;
};

export const listFiles = (root: string, rel: string) =>
    invoke<FileEntry[]>("list_files", { root, rel });
export const readFile = (root: string, rel: string) =>
    invoke<string>("read_file", { root, rel });
export const writeFile = (root: string, rel: string, content: string) =>
    invoke("write_file", { root, rel, content });
export const createFile = (root: string, rel: string, name: string) =>
    invoke<string>("create_file", { root, rel, name });
