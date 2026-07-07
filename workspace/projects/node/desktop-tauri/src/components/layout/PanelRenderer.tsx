import React from "react";
import { PanelLayout, PanelWidget, WidgetKind } from "../../types";

interface PanelRendererProps {
  layout?: PanelLayout;
}

export function PanelRenderer({ layout }: PanelRendererProps) {
  if (!layout) {
    return (
      <div className="flex items-center justify-center h-full text-gray-500 text-sm">
        No custom panel layout provided by the plugin.
      </div>
    );
  }

  return (
    <div className="flex flex-col gap-6 p-6 h-full overflow-y-auto bg-[#181824] text-gray-200">
      <div className="flex items-center justify-between border-b border-gray-800 pb-3">
        <h2 className="text-lg font-semibold tracking-wide text-blue-400">
          {layout.title}
        </h2>
      </div>

      <div className="grid grid-cols-1 gap-6">
        {layout.widgets.map((widget, idx) => (
          <WidgetComponent key={idx} widget={widget} />
        ))}
      </div>
    </div>
  );
}

function WidgetComponent({ widget }: { widget: PanelWidget }) {
  return (
    <div className="flex flex-col gap-3 p-4 rounded-lg bg-[#1f1f2e] border border-gray-800/80 shadow-sm">
      <h3 className="text-xs font-semibold uppercase tracking-wider text-gray-400 border-b border-gray-800/50 pb-2">
        {widget.title}
      </h3>
      <div className="mt-1">
        {renderWidgetContent(widget.widget_kind, widget.data)}
      </div>
    </div>
  );
}

function renderWidgetContent(kind: WidgetKind, data: string[][]): React.ReactNode {
  if (data.length === 0) {
    return <div className="text-gray-500 text-xs italic">No data</div>;
  }

  switch (kind) {
    case "Table": {
      const headers = data[0];
      const rows = data.slice(1);
      return (
        <div className="overflow-x-auto w-full border border-gray-800 rounded-md">
          <table className="w-full text-left border-collapse text-xs">
            <thead>
              <tr className="bg-[#2a2a3d] border-b border-gray-800">
                {headers.map((h, i) => (
                  <th key={i} className="p-3 font-semibold text-blue-300">
                    {h}
                  </th>
                ))}
              </tr>
            </thead>
            <tbody>
              {rows.map((row, rIdx) => (
                <tr
                  key={rIdx}
                  className="border-b border-gray-800/60 last:border-0 hover:bg-[#252538] transition-colors"
                >
                  {row.map((cell, cIdx) => (
                    <td key={cIdx} className="p-3 text-gray-300 font-mono">
                      {cell}
                    </td>
                  ))}
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      );
    }

    case "KeyValue":
      return (
        <div className="grid grid-cols-1 md:grid-cols-2 gap-3 text-xs">
          {data.map((row, idx) => {
            const key = row[0] || "";
            const value = row[1] || "";
            return (
              <div
                key={idx}
                className="flex flex-col sm:flex-row sm:items-center justify-between p-2 rounded bg-[#252538] border border-gray-800/40"
              >
                <span className="font-semibold text-gray-400">{key}</span>
                <span className="font-mono text-gray-200 mt-1 sm:mt-0 text-right">
                  {value}
                </span>
              </div>
            );
          })}
        </div>
      );

    case "Tree":
    case "Log":
      return (
        <div className="p-3 rounded bg-[#151520] border border-gray-900 font-mono text-xs text-gray-400 leading-relaxed max-h-60 overflow-y-auto space-y-1">
          {data.map((row, idx) => (
            <div key={idx} className="hover:text-gray-200 transition-colors">
              {row.join(" ")}
            </div>
          ))}
        </div>
      );

    case "ChartBar":
      return (
        <div className="space-y-3 text-xs">
          {data.map((row, idx) => {
            const label = row[0] || "Unknown";
            const valStr = row[1] || "0";
            const value = parseFloat(valStr) || 0;
            // Bound between 0 and 100 for percentage visualization
            const percentage = Math.min(Math.max(value, 0), 100);

            return (
              <div key={idx} className="flex items-center gap-4">
                <span className="w-24 text-gray-400 font-semibold truncate text-right">
                  {label}
                </span>
                <div className="flex-1 h-3 rounded bg-gray-800 overflow-hidden relative border border-gray-700/50">
                  <div
                    className="h-full bg-gradient-to-r from-blue-500 to-indigo-600 rounded-r"
                    style={{ width: `${percentage}%` }}
                  />
                </div>
                <span className="w-10 text-gray-300 font-mono text-right font-semibold">
                  {valStr}
                </span>
              </div>
            );
          })}
        </div>
      );

    case "TextBlock":
    default:
      return (
        <div className="space-y-2 text-xs text-gray-300 leading-relaxed font-sans">
          {data.map((row, idx) => (
            <p key={idx} className="indent-2">
              {row.join(" ")}
            </p>
          ))}
        </div>
      );
  }
}
