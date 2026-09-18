export interface GithubWorkflow {
  id: number;
  name: string;
  path: string;
  state: string;
  html_url?: string;
}

export interface GithubWorkflowRun {
  id: number;
  name?: string;
  head_branch?: string;
  head_sha?: string;
  event: string;
  status: string;
  conclusion?: string;
  html_url: string;
  created_at: string;
  updated_at: string;
  run_number: number;
  actor_login?: string;
  actor_avatar_url?: string;
}

export interface GithubStep {
  name: string;
  status: string;
  conclusion?: string;
  number: number;
  started_at?: string;
  completed_at?: string;
}

export interface GithubJob {
  id: number;
  run_id: number;
  name: string;
  status: string;
  conclusion?: string;
  started_at?: string;
  completed_at?: string;
  steps: GithubStep[];
}
