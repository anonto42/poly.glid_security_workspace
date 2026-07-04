export interface Issue {
  title: string;
  severity: string;
  description: string;
  recommendation: string;
}

export type WidgetKind = 'Table' | 'KeyValue' | 'Tree' | 'Log' | 'ChartBar' | 'TextBlock';

export interface PanelWidget {
  widget_kind: WidgetKind;
  title: string;
  data: string[][];
}

export interface PanelLayout {
  title: string;
  widgets: PanelWidget[];
}

export interface Report {
  plugin_name: string;
  target_tested: string;
  issues: Issue[];
  summary: string;
  panel?: PanelLayout;
}

export interface PluginInfo {
  id: string;
  name: string;
  version: string;
  author: string;
  description: string;
  capabilities: string[];
  checksum: string;
  status: 'Enabled' | 'Disabled' | 'Invalid' | 'UpdateAvailable';
  source: string;
  file_size: number;
  installed_at: number;
  last_updated: number;
  path: string;
}
