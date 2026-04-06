import { t, locale } from "../i18n";
import { toggleFavourite, isFavourite } from "../lib/state";
import { formatTime } from "../lib/time";
import type { Program } from "../lib/types";

interface Props {
  program: Program;
  x: number;
  y: number;
  onClose: () => void;
}

export function ProgramPopup(props: Props) {
  const handleToggle = async () => {
    await toggleFavourite(props.program.title);
  };

  const style = () => {
    const x = Math.min(props.x, window.innerWidth - 340);
    const y = Math.min(props.y + 10, window.innerHeight - 250);
    return `left:${x}px;top:${y}px`;
  };

  return (
    <>
      <div style="position:fixed;inset:0;z-index:99" onClick={props.onClose} />
      <div class="program-popup" style={style()}>
        <h3>{props.program.title}</h3>
        <div class="time">
          {formatTime(props.program.start_time, locale())} –{" "}
          {formatTime(props.program.end_time, locale())}
        </div>
        {props.program.description && (
          <div class="description">{props.program.description}</div>
        )}
        {props.program.category && (
          <div class="category">{props.program.category}</div>
        )}
        <button
          class={`star-btn ${isFavourite(props.program.title) ? "active" : ""}`}
          onClick={handleToggle}
        >
          {isFavourite(props.program.title) ? "\u2605" : "\u2606"}{" "}
          {isFavourite(props.program.title) ? t("remove_favourite") : t("add_favourite")}
        </button>
      </div>
    </>
  );
}
