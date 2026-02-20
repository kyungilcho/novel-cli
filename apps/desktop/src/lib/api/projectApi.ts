import { invoke } from "@tauri-apps/api/core";

export type ProjectInfo = {
    root: string;
    name: string;
};

export const openProject = (root: string) =>
    invoke<ProjectInfo>("open_project", { root });

export const defaultWorkspaceRoot = () =>
    invoke<string>("default_workspace_root");
