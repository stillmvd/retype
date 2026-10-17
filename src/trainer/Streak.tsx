import { plural } from "../api";
import { Icon } from "../Icon";

export function Streak({ n }: { n: number }) {
  if (n <= 0) return null;
  return <span className="tr-streak"><Icon name="flame" /><b>{n} {plural(n, "день", "дня", "дней")}</b> подряд</span>;
}
