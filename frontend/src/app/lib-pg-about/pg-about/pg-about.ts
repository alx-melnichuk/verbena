import { CommonModule } from "@angular/common";
import { Component, ViewEncapsulation, ChangeDetectionStrategy, inject } from "@angular/core";
import { takeUntilDestroyed } from "@angular/core/rxjs-interop";
import { RouterOutlet } from "@angular/router";
import { TranslateService, LangChangeEvent } from "@ngx-translate/core";
import { environment as env } from "../../../environments/environment";
import { LocaleSrv } from "../../common/locale-srv";
import { LOG_PG_ABOUT } from "../../common/routes";
import { AlertSrv } from "../../lib-dialog/alert-srv";
import { DialogSrv } from "../../lib-dialog/dialog-srv";

@Component({
    selector: "app-pg-about",
    exportAs: "appPgAbout",
    standalone: true,
    imports: [CommonModule, RouterOutlet],
    templateUrl: "./pg-about.html",
    styleUrl: "./pg-about.scss",
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush,
    providers: [ // Create a separate instance of "AlertSrv" to access translations of the current module.
        { provide: AlertSrv, useClass: AlertSrv },
        { provide: DialogSrv, useClass: DialogSrv },
    ],
})
export class PgAbout {
    private localeSrv: LocaleSrv = inject(LocaleSrv);
    private translateSrv: TranslateService = inject(TranslateService);

    // The current locale is loaded in the resolver.
    private loadedLangs: Record<string, boolean> = { [this.localeSrv.getLocale()]: true };

    private langChange$ = this.localeSrv.onLangChange.pipe(takeUntilDestroyed())
        .subscribe((event: LangChangeEvent) => {
            const title = `PgAbout().onLangChange(${event.lang})`;
            if (!this.loadedLangs[event.lang] && !!this.translateSrv.translations) {
                if (env.logLevel & LOG_PG_ABOUT) {
                    console.info(`${title} translate.setTranslation(${event.lang}); trans:`, { ...event.translations });
                }
                this.loadedLangs[event.lang] = true;
                // Add translations from the main module to this module.
                this.translateSrv.setTranslation(event.lang, event.translations, true);
            }
            this.translateSrv.use(event.lang);
        });

    constructor() {
        if (env.logLevel & LOG_PG_ABOUT) { console.info(`PgAbout(); locale.getLocale(): ${this.localeSrv.getLocale()}`); }
    }

    // ** Public API **

    // ** Private API **
}
