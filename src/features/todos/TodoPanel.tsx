import { useEffect, useState } from "react";
import type { Todo } from "@/types/tauriCommands";
import * as todoService from "@/services/todoService";

export function TodoPanel({ projectId }: { projectId: string }) {
  const [todos, setTodos] = useState<Todo[]>([]);
  const [newTitle, setNewTitle] = useState("");

  useEffect(() => {
    todoService.listTodos(projectId).then(setTodos);
  }, [projectId]);

  async function handleAdd(e: React.FormEvent) {
    e.preventDefault();
    const title = newTitle.trim();
    if (!title) return;
    const created = await todoService.createTodo(projectId, title);
    setTodos((ts) => [...ts, created]);
    setNewTitle("");
  }

  async function toggle(todo: Todo) {
    const updated = await todoService.setTodoDone(todo.id, !todo.done);
    setTodos((ts) => ts.map((t) => (t.id === updated.id ? updated : t)));
  }

  async function handleDelete(id: string) {
    await todoService.deleteTodo(id);
    setTodos((ts) => ts.filter((t) => t.id !== id));
  }

  const remaining = todos.filter((t) => !t.done).length;

  return (
    <div className="section-layout">
      <form className="field-row" style={{ padding: "10px 12px" }} onSubmit={handleAdd}>
        <input value={newTitle} onChange={(e) => setNewTitle(e.target.value)} placeholder="新しいTODO" />
        <button className="primary" type="submit">
          追加
        </button>
      </form>
      <p className="status-line" style={{ padding: "0 12px" }}>
        未完了 {remaining} / 全{todos.length}件
      </p>
      <ul className="todo-list">
        {todos.map((todo) => (
          <li key={todo.id} className="todo-list__item">
            <input type="checkbox" checked={todo.done} onChange={() => toggle(todo)} aria-label={`${todo.title}を完了にする`} />
            <span className={`todo-list__title ${todo.done ? "todo-list__title--done" : ""}`}>{todo.title}</span>
            <button className="secondary" onClick={() => handleDelete(todo.id)}>
              削除
            </button>
          </li>
        ))}
        {todos.length === 0 && <li className="status-line">TODOはありません。</li>}
      </ul>
    </div>
  );
}
