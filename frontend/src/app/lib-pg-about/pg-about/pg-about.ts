import { CommonModule } from "@angular/common";
import { Component, ViewEncapsulation, ChangeDetectionStrategy, inject } from "@angular/core";
import { takeUntilDestroyed } from "@angular/core/rxjs-interop";
import { TranslateService, LangChangeEvent } from "@ngx-translate/core";
import { environment } from "../../../environments/environment";
import { LocaleSrv } from "../../common/locale-srv";
import { PanelAbout } from "../panel-about/panel-about";

declare var APP_ABOUT: any;

@Component({
    selector: "app-pg-about",
    exportAs: "appPgAbout",
    standalone: true,
    imports: [CommonModule, PanelAbout],
    templateUrl: "./pg-about.html",
    styleUrl: "./pg-about.scss",
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class PgAbout {
    private translate = inject(TranslateService);
    private locale = inject(LocaleSrv);

    public get appAbout(): any { return APP_ABOUT || {}; }
    public set appAbout(_val: any) { }

    public backendItem01 = this.appAbout["backend01"] || "";
    public backendItem02: string[] = this.appAbout["backend02"] || [];
    public backendItem03: string[] = this.appAbout["backend03"] || [];

    // The current locale is loaded in the resolver.
    private loadedLangs: Record<string, boolean> = { [this.locale.getLocale()]: true };

    private langChange$ = this.locale.onLangChange.pipe(takeUntilDestroyed())
        .subscribe((event: LangChangeEvent) => {
            if (!this.loadedLangs[event.lang] && !!this.translate.translations) {
                if (environment.logLevel > 0) {
                    console.info(`PgAbout().onLangChange(${event.lang}) translate.setTranslation(${event.lang}); trans:`
                        , { ...event.translations });
                }
                this.loadedLangs[event.lang] = true;
                // Add translations from the main module to this module.
                this.translate.setTranslation(event.lang, event.translations, true);
            }
            this.translate.use(event.lang);
        });

    constructor() {
        if (environment.logLevel > 0) { console.info(`PgAbout(); locale.getLocale(): ${this.locale.getLocale()}`); }
    }

    // ** Public API **

    // ** Private API **
}
