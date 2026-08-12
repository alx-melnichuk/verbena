import { CommonModule } from "@angular/common";
import { HttpErrorResponse } from "@angular/common/http";
import { ChangeDetectionStrategy, ChangeDetectorRef, Component, inject, ViewEncapsulation } from "@angular/core";
import { takeUntilDestroyed } from "@angular/core/rxjs-interop";
import { Router } from "@angular/router";
import { TranslateService, LangChangeEvent } from "@ngx-translate/core";
import { environment } from "../../../environments/environment";
import { LocaleSrv } from "../../common/locale-srv";
import { ROUTE_LOGIN } from "../../common/routes";
import { AlertSrv } from "../../lib-dialog/alert-srv";
import { DialogSrv } from "../../lib-dialog/dialog-srv";
import { UserSrv } from "../../lib-user/user-srv";
import { ErrMsgObj, HttpErrorUtil } from "../../utils/http-error.util";
import { PanelForgotPassword } from "../panel-forgot-password/panel-forgot-password";

@Component({
    selector: "app-pg-forgot-password",
    exportAs: "appPgLogin",
    standalone: true,
    imports: [CommonModule, PanelForgotPassword],
    templateUrl: "./pg-forgot-password.html",
    styleUrl: "./pg-forgot-password.scss",
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush,
    providers: [ // Create a separate instance of "AlertSrv" to access translations of the current module.
        { provide: AlertSrv, useClass: AlertSrv },
        { provide: DialogSrv, useClass: DialogSrv },
    ],
})
export class PgForgotPassword {
    private changeDetector: ChangeDetectorRef = inject(ChangeDetectorRef);
    private dialogSrv: DialogSrv = inject(DialogSrv);
    private localeSrv: LocaleSrv = inject(LocaleSrv);
    private userSrv: UserSrv = inject(UserSrv);
    private router: Router = inject(Router);
    private translateSrv: TranslateService = inject(TranslateService);

    public errMsgObjs: ErrMsgObj[] = [];
    public isDisabledSubmit = false;

    // The current locale is loaded in the resolver.
    private loadedLangs: Record<string, boolean> = { [this.localeSrv.getLocale()]: true };

    private langChange$ = this.localeSrv.onLangChange.pipe(takeUntilDestroyed())
        .subscribe((event: LangChangeEvent) => {
            const title = `PgForgotPassword().onLangChange(${event.lang})`;
            if (!this.loadedLangs[event.lang] && !!this.translateSrv.translations) {
                if (environment.logLevel > 0) {
                    console.info(`${title} translate.setTranslation(${event.lang}); trans:`, { ...event.translations });
                }
                this.loadedLangs[event.lang] = true;
                // Add translations from the main module to this module.
                this.translateSrv.setTranslation(event.lang, event.translations, true);
            }
            this.translateSrv.use(event.lang);
        });

    constructor() {
        if (environment.logLevel > 0) { console.info(`PgForgotPassword(); locale.getLocale(): ${this.localeSrv.getLocale()}`); }
    }

    // ** Public API **

    public doResend(params: Record<string, (string | null)>): void {
        if (!params) {
            return;
        }
        const email: string = params["email"] || "";
        if (!email) {
            return;
        }

        this.isDisabledSubmit = true;
        this.errMsgObjs = [];
        this.userSrv.recovery(email)
            .then(() => {
                const appName = this.translateSrv.instant("app.name");
                const title = this.translateSrv.instant("pg-forgot-password.dialog_title", { appName: appName });
                const message = this.translateSrv.instant("pg-forgot-password.dialog_message", { value: email });
                this.dialogSrv.openConfirmation(message, title, { btnNameAccept: "buttons.ok" }).then(() => {
                    window.setTimeout(() => this.router.navigateByUrl(ROUTE_LOGIN, { replaceUrl: true }), 0);
                });
            })
            .catch((err: HttpErrorResponse) => {
                this.errMsgObjs = HttpErrorUtil.mapErrMsgObjs(err.status, err.error);
                throw err;
            })
            .finally(() => {
                this.isDisabledSubmit = false;
                this.changeDetector.markForCheck();
            });
    }

    // ** Private API **

}
