import { CommonModule } from "@angular/common";
import { ChangeDetectionStrategy, Component, inject, ViewEncapsulation } from "@angular/core";
import { takeUntilDestroyed } from "@angular/core/rxjs-interop";
import { RouterOutlet } from "@angular/router";
import { TranslateService, LangChangeEvent } from "@ngx-translate/core";
import { environment } from "../../../environments/environment";
import { LocaleSrv } from "../../common/locale-srv";

@Component({
    selector: "app-pg-stream",
    exportAs: "appPgStream",
    standalone: true,
    imports: [CommonModule, RouterOutlet],
    templateUrl: "./pg-stream.html",
    styleUrl: "./pg-stream.scss",
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class PgStream {
    private localeSrv: LocaleSrv = inject(LocaleSrv);
    private translate: TranslateService = inject(TranslateService);

    // The current locale is loaded in the resolver.
    private loadedLangs: Record<string, boolean> = { [this.localeSrv.getLocale()]: true };

    private langChange$ = this.localeSrv.onLangChange.pipe(takeUntilDestroyed())
        .subscribe((event: LangChangeEvent) => {
            const title = `PgStream().onLangChange(${event.lang})`;
            if (!this.loadedLangs[event.lang] && !!this.translate.translations) {
                if (environment.logLevel > 0) {
                    console.info(`${title} translate.setTranslation(${event.lang}); trans:`, { ...event.translations });
                }
                this.loadedLangs[event.lang] = true;
                // Add translations from the main module to this module.
                this.translate.setTranslation(event.lang, event.translations, true);
            }
            this.translate.use(event.lang);
        });

    constructor() {
        if (environment.logLevel > 0) { console.info(`PgStream(); locale.getLocale(): ${this.localeSrv.getLocale()}`); }
    }

    // ** Public API **

    // ** Private API **

}
