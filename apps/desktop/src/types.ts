export interface Issue {
  title: string;
  severity: string;
  description: string;
  recommendation: string;
}

export interface Report {
  plugin_name: string;
  target_tested: string;
  issues: Issue[];
  summary: string;
}

export interface PluginInfo {
  name: string;
  path: string;
}
