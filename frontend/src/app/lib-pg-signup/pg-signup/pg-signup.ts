import { CommonModule } from "@angular/common";
import { HttpErrorResponse } from "@angular/common/http";
import { ChangeDetectionStrategy, ChangeDetectorRef, Component, inject, ViewEncapsulation } from "@angular/core";
import { takeUntilDestroyed } from "@angular/core/rxjs-interop";
import { Router } from "@angular/router";
import { TranslateService, LangChangeEvent } from "@ngx-translate/core";
import { environment as env } from "../../../environments/environment";
import { LocaleSrv } from "../../common/locale-srv";
import { LOG_PG_SIGNUP, ROUTE_LOGIN } from "../../common/routes";
import { AlertSrv } from "../../lib-dialog/alert-srv";
import { DialogSrv } from "../../lib-dialog/dialog-srv";
import { UserSrv } from "../../lib-user/user-srv";
import { ErrMsgObj, HttpErrorUtil } from "../../utils/http-error.util";
import { PanelSignup } from "../panel-signup/panel-signup";

@Component({
    selector: "app-pg-signup",
    exportAs: "appPgLogin",
    standalone: true,
    imports: [CommonModule, PanelSignup],
    templateUrl: "./pg-signup.html",
    styleUrl: "./pg-signup.scss",
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush,
    providers: [ // Create a separate instance of "AlertSrv" to access translations of the current module.
        { provide: AlertSrv, useClass: AlertSrv },
        { provide: DialogSrv, useClass: DialogSrv },
    ],
})
export class PgSignup {
    private changeDetector: ChangeDetectorRef = inject(ChangeDetectorRef);
    private dialogSrv: DialogSrv = inject(DialogSrv);
    private localeSrv: LocaleSrv = inject(LocaleSrv);
    private router: Router = inject(Router);
    private userSrv: UserSrv = inject(UserSrv);
    private translateSrv: TranslateService = inject(TranslateService);

    public errMsgObjs: ErrMsgObj[] = [];
    public isLogin = true;
    public isDisabled = false;

    // The current locale is loaded in the resolver.
    private loadedLangs: Record<string, boolean> = { [this.localeSrv.getLocale()]: true };

    private langChange$ = this.localeSrv.onLangChange.pipe(takeUntilDestroyed())
        .subscribe((event: LangChangeEvent) => {
            const title = `PgSignup().onLangChange(${event.lang})`;
            if (!this.loadedLangs[event.lang] && !!this.translateSrv.translations) {
                if (env.logLevel & LOG_PG_SIGNUP) {
                    console.info(`${title} translate.setTranslation(${event.lang}); trans:`, { ...event.translations });
                }
                this.loadedLangs[event.lang] = true;
                // Add translations from the main module to this module.
                this.translateSrv.setTranslation(event.lang, event.translations, true);
            }
            this.translateSrv.use(event.lang);
        });

    constructor() {
        if (env.logLevel & LOG_PG_SIGNUP) { console.info(`PgSignup(); locale.getLocale(): ${this.localeSrv.getLocale()}`); }
    }

    // ** Public API **

    public doSignup(params: Record<string, string>): void {
        if (!params) {
            return;
        }
        const nickname: string = params["nickname"] || "";
        const password: string = params["password"] || "";
        const email: string = params["email"] || "";

        if (!nickname || !password || !email) {
            return;
        }

        this.isDisabled = true;
        this.errMsgObjs = [];
        this.userSrv.registration(nickname, email, password)
            .then(() => {
                const appName = this.translateSrv.instant("app.name");
                const title = this.translateSrv.instant("pg-signup.dialog_title", { appName: appName });
                const message = this.translateSrv.instant("pg-signup.dialog_message", { value: email });
                this.dialogSrv.openConfirmation(message, title, { btnNameAccept: "buttons.ok" })
                    .then(() => {
                        window.setTimeout(() => this.router.navigateByUrl(ROUTE_LOGIN, { replaceUrl: true }), 0);
                    });
            })
            .catch((err: HttpErrorResponse) => {
                this.errMsgObjs = HttpErrorUtil.mapErrMsgObjs(err.status, err.error);
            })
            .finally(() => {
                this.isDisabled = false;
                this.changeDetector.markForCheck();
            });
    }

    // ** Private API **

}
