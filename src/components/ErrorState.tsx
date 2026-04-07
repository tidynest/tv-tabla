import { t } from "../i18n";
import { refreshData } from "../lib/state";

export function ErrorState() {
  return (
    <div class="error-state">
      <p>{t("no_connection")}</p>
      <button onClick={() => refreshData()}>{t("retry")}</button>
    </div>
  );
}
