import { Route, Routes } from "react-router-dom";
import { ProjectListPage } from "@/features/projects/ProjectListPage";
import { ProjectWorkspacePage } from "@/features/projects/ProjectWorkspacePage";

/**
 * ルート定義のみ。`HashRouter`自体は`App.tsx`側で、アプリシェル全体
 * (トップバー・Command Palette等)を包む位置に置いている(Phase11、
 * Command Paletteが`useNavigate`/`useParams`等のRouterコンテキストを
 * ルート要素の外からも使えるようにするため)。
 */
export function AppRouter() {
  return (
    <Routes>
      <Route path="/" element={<ProjectListPage />} />
      <Route path="/projects/:projectId" element={<ProjectWorkspacePage />} />
    </Routes>
  );
}
