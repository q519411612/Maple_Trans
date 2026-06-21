import { useMemo, useState } from "react";
import {
  canInstall,
  checkTranslationData,
  ClientStatus,
  inspectGameDir,
  installLocalization,
  LanguageMode,
  restoreOriginal,
} from "./api";

type Page = "install" | "restore" | "data" | "logs" | "settings" | "about";

const pages: Array<{ id: Page; label: string }> = [
  { id: "install", label: "安装" },
  { id: "restore", label: "还原" },
  { id: "data", label: "翻译数据" },
  { id: "logs", label: "日志" },
  { id: "settings", label: "设置" },
  { id: "about", label: "关于" },
];

export function App() {
  const [page, setPage] = useState<Page>("install");
  const [gameDir, setGameDir] = useState("");
  const [mode, setMode] = useState<LanguageMode>("zh-CN");
  const [status, setStatus] = useState<ClientStatus>({ kind: "notSelected" });
  const [logs, setLogs] = useState<string[]>([]);
  const installEnabled = canInstall(status);

  const statusText = useMemo(() => {
    if (status.kind === "notSelected") return "尚未选择游戏目录";
    if (status.kind === "unsupported") return status.reason;
    return `已支持版本 ${status.version}${status.installed ? "，已安装补丁" : "，未安装补丁"}`;
  }, [status]);

  async function inspect() {
    try {
      const next = await inspectGameDir(gameDir);
      setStatus(next);
      addLog(`检查目录：${statusToLog(next)}`);
    } catch (error) {
      addLog(`检查失败：${String(error)}`);
      setStatus({ kind: "unsupported", reason: String(error) });
    }
  }

  async function install() {
    try {
      const result = await installLocalization(gameDir, mode);
      addLog(result.message);
    } catch (error) {
      addLog(`安装失败：${String(error)}`);
    }
  }

  async function restore() {
    try {
      const result = await restoreOriginal(gameDir);
      addLog(result.message);
    } catch (error) {
      addLog(`还原失败：${String(error)}`);
    }
  }

  async function updateData() {
    try {
      const result = await checkTranslationData();
      addLog(result.message);
    } catch (error) {
      addLog(`更新检查失败：${String(error)}`);
    }
  }

  function addLog(message: string) {
    setLogs((current) => [`${new Date().toLocaleTimeString()} ${message}`, ...current].slice(0, 50));
  }

  return (
    <main className="app-shell">
      <aside className="sidebar">
        <h1>Open Maple Patch</h1>
        <p>非官方文本汉化助手</p>
        <nav>
          {pages.map((item) => (
            <button
              key={item.id}
              className={page === item.id ? "active" : ""}
              onClick={() => setPage(item.id)}
              type="button"
            >
              {item.label}
            </button>
          ))}
        </nav>
      </aside>

      <section className="content">
        {page === "install" && (
          <section>
            <div className="section-heading">
              <p>安装</p>
              <h2>选择 MapleLegends 文件夹</h2>
            </div>
            <div className="field-row">
              <input
                value={gameDir}
                onChange={(event) => setGameDir(event.target.value)}
                placeholder="C:\\Games\\MapleLegends"
              />
              <button type="button" onClick={inspect} disabled={!gameDir}>
                检查
              </button>
            </div>
            <label className="select-row">
              显示模式
              <select value={mode} onChange={(event) => setMode(event.target.value as LanguageMode)}>
                <option value="zh-CN">简体中文</option>
                <option value="zh-CN-bilingual">中英双语</option>
              </select>
            </label>
            <StatusPanel status={status} text={statusText} />
            <div className="actions">
              <button type="button" onClick={install} disabled={!installEnabled}>
                安装汉化
              </button>
              <button type="button" onClick={restore} disabled={!gameDir}>
                还原原版
              </button>
            </div>
          </section>
        )}

        {page === "restore" && (
          <section>
            <div className="section-heading">
              <p>还原</p>
              <h2>恢复备份的原始文件</h2>
            </div>
            <StatusPanel status={status} text={statusText} />
            <button type="button" onClick={restore} disabled={!gameDir}>
              还原原版
            </button>
          </section>
        )}

        {page === "data" && (
          <section>
            <div className="section-heading">
              <p>翻译数据</p>
              <h2>只检查和更新翻译数据</h2>
            </div>
            <p className="notice">此操作不会更新程序本体，也不会下载 MapleLegends 客户端文件。</p>
            <button type="button" onClick={updateData}>
              检查翻译数据
            </button>
          </section>
        )}

        {page === "logs" && (
          <section>
            <div className="section-heading">
              <p>日志</p>
              <h2>最近操作</h2>
            </div>
            <pre className="logs">{logs.length ? logs.join("\n") : "暂无日志"}</pre>
          </section>
        )}

        {page === "settings" && (
          <section>
            <div className="section-heading">
              <p>设置</p>
              <h2>语言和诊断选项</h2>
            </div>
            <label className="select-row">
              默认显示模式
              <select value={mode} onChange={(event) => setMode(event.target.value as LanguageMode)}>
                <option value="zh-CN">简体中文</option>
                <option value="zh-CN-bilingual">中英双语</option>
              </select>
            </label>
          </section>
        )}

        {page === "about" && (
          <section>
            <div className="section-heading">
              <p>关于</p>
              <h2>非官方文本汉化补丁器</h2>
            </div>
            <p className="notice">
              本项目不隶属于 MapleLegends 或 Nexon。它不分发客户端文件，不修改玩法数值、地图碰撞、角色状态、网络包、内存或自动化行为。使用风险由使用者自行承担。
            </p>
          </section>
        )}
      </section>
    </main>
  );
}

function StatusPanel({ status, text }: { status: ClientStatus; text: string }) {
  return (
    <div className={`status-panel ${status.kind}`}>
      <strong>{text}</strong>
      {status.kind === "supported" && status.changedFiles.length > 0 && (
        <ul>
          {status.changedFiles.map((file) => (
            <li key={file}>{file}</li>
          ))}
        </ul>
      )}
    </div>
  );
}

function statusToLog(status: ClientStatus): string {
  if (status.kind === "notSelected") return "未选择";
  if (status.kind === "unsupported") return `不支持：${status.reason}`;
  return `支持 ${status.version}`;
}
