import { invoke } from "@tauri-apps/api/core";

export type RepoState = {
    head: string | null;
    node_count: number;
};

export type VersionNode = {
    id: string;
    parents: string[];
    message: string;
    created_at_unix_ms: number;
};

export const initRepo = (root: string) => invoke<void>("init_repo", { root });

export const fetchRepoState = (root: string) =>
    invoke<RepoState>("repo_state", { root });

export const fetchLog = (root: string) =>
    invoke<VersionNode[]>("log", { root });

export const commitSnapshot = (root: string, message: string) =>
    invoke<string>("commit", { root, message });

export const checkoutSnapshot = (root: string, nodeId: string) =>
    invoke<void>("checkout", { root, nodeId, node_id: nodeId });
