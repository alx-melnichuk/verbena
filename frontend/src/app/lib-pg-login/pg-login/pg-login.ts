import { CommonModule } from "@angular/common";
import { HttpErrorResponse } from "@angular/common/http";
import { Component, ViewEncapsulation, ChangeDetectionStrategy, inject, ChangeDetectorRef } from "@angular/core";
import { takeUntilDestroyed } from "@angular/core/rxjs-interop";
import { Router } from "@angular/router";
import { TranslateService, LangChangeEvent } from "@ngx-translate/core";
import { environment as env } from "../../../environments/environment";
import { LocaleSrv } from "../../common/locale-srv";
import { RedirectSrv } from "../../common/redirect-srv";
import { LOG_PG_LOGIN, REDIRECT_AFTER_LOGIN } from "../../common/routes";
import { SessionSrv } from "../../common/session-srv";
import { AlertSrv } from "../../lib-dialog/alert-srv";
import { DialogSrv } from "../../lib-dialog/dialog-srv";
import { LoginResponseDto } from "../../lib-user/user-dto";
import { UserSrv } from "../../lib-user/user-srv";
import { ErrMsgObj, HttpErrorUtil } from "../../utils/http-error.util";
import { PanelLogin } from "../panel-login/panel-login";

@Component({
    selector: "app-pg-login",
    exportAs: "appPgLogin",
    standalone: true,
    imports: [CommonModule, PanelLogin],
    templateUrl: "./pg-login.html",
    styleUrl: "./pg-login.scss",
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush,
    providers: [ // Create a separate instance of "AlertSrv" to access translations of the current module.
        { provide: AlertSrv, useClass: AlertSrv },
        { provide: DialogSrv, useClass: DialogSrv },
    ],
})
export class PgLogin {
    private changeDetector: ChangeDetectorRef = inject(ChangeDetectorRef);
    private localeSrv: LocaleSrv = inject(LocaleSrv);
    private redirectSrv: RedirectSrv = inject(RedirectSrv);
    private router: Router = inject(Router);
    private sessionSrv: SessionSrv = inject(SessionSrv);
    private userSrv: UserSrv = inject(UserSrv);
    private translateSrv: TranslateService = inject(TranslateService);

    public errMsgObjs: ErrMsgObj[] = [];
    public isLogin = true;
    public isDisabledSubmit = false;

    // The current locale is loaded in the resolver.
    private loadedLangs: Record<string, boolean> = { [this.localeSrv.getLocale()]: true };

    private langChange$ = this.localeSrv.onLangChange.pipe(takeUntilDestroyed())
        .subscribe((event: LangChangeEvent) => {
            const title = `PgLogin().onLangChange(${event.lang})`;
            if (!this.loadedLangs[event.lang] && !!this.translateSrv.translations) {
                if (env.logLevel & LOG_PG_LOGIN) {
                    console.info(`${title} translate.setTranslation(${event.lang}); trans:`, { ...event.translations });
                }
                this.loadedLangs[event.lang] = true;
                // Add translations from the main module to this module.
                this.translateSrv.setTranslation(event.lang, event.translations, true);
            }
            this.translateSrv.use(event.lang);
            // #.pipe(first())
            // #.subscribe({ next: () => env.logLevel & LOG_PG ? console.info(`${title} translate.use(${event.lang})...Ok`) : "" });
        });

    constructor() {
        if (env.logLevel & LOG_PG_LOGIN) { console.info(`PgLogin(); locale.getLocale(): ${this.localeSrv.getLocale()}`); }
    }

    // ** Public API **

    public doLogin(params: Record<string, (string | null)>): void {
        if (!params) {
            return;
        }
        const nickname: string = params["nickname"] || "";
        const password: string = params["password"] || "";

        if (!nickname || !password) {
            return;
        }
        this.isDisabledSubmit = true;
        this.errMsgObjs = [];
        this.sessionSrv.removeUserTokens();
        this.sessionSrv.removeUser();

        this.userSrv.login(nickname, password)
            .then((response: LoginResponseDto | HttpErrorResponse | undefined) => {
                let res: LoginResponseDto = response as LoginResponseDto;
                this.sessionSrv.setUser(res.userProfileDto);
                this.sessionSrv.setUserTokens(res.tokenUserResponseDto.accessToken, res.tokenUserResponseDto.refreshToken);

                Promise.resolve().then(() => {
                    // Get the link address to navigate to after login.
                    const urlAfterLogin = this.redirectSrv.getUrlAfterLogin() || REDIRECT_AFTER_LOGIN;
                    this.redirectSrv.setUrlAfterLogin("");
                    this.router.navigateByUrl(urlAfterLogin);
                });
            })
            .catch((err: HttpErrorResponse) => {
                this.errMsgObjs = HttpErrorUtil.mapErrMsgObjs(err.status, err.error);
                throw err;
            })
            .finally(() => {
                this.isDisabledSubmit = false;
                this.changeDetector.markForCheck();
            })
    }

    // ** Private API **

}
