import { CommonModule } from "@angular/common";
import { HttpErrorResponse } from "@angular/common/http";
import { ChangeDetectionStrategy, ChangeDetectorRef, Component, inject, Renderer2, ViewEncapsulation } from "@angular/core";
import { takeUntilDestroyed } from "@angular/core/rxjs-interop";
import { ActivatedRoute, Router } from "@angular/router";
import { TranslateService, LangChangeEvent } from "@ngx-translate/core";
import { environment } from "../../../environments/environment";
import { LocaleSrv } from "../../common/locale-srv";
import { ROUTE_LOGIN } from "../../common/routes";
import { SessionSrv } from "../../common/session-srv";
import { Spinner } from "../../components/spinner/spinner";
import { AlertSrv } from "../../lib-dialog/alert-srv";
import { DialogSrv } from "../../lib-dialog/dialog-srv";
import { ErrMsgObj, HttpErrorUtil } from "../../utils/http-error.util";
import { PanelProfile } from "../panel-profile/panel-profile";
import { ProfileConfigDto } from "../profile-config-dto";
import { ProfileDto, ModifyProfileDto, NewPasswordProfileDto } from "../profile-dto";
import { ProfileSrv } from "../profile-srv";

@Component({
    selector: "app-pg-profile",
    exportAs: "appPgProfile",
    standalone: true,
    imports: [CommonModule, PanelProfile, Spinner],
    templateUrl: "./pg-profile.html",
    styleUrl: "./pg-profile.scss",
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush,
    providers: [ // Create a separate instance of "AlertSrv" to access translations of the current module.
        { provide: AlertSrv, useClass: AlertSrv },
        { provide: DialogSrv, useClass: DialogSrv },
    ],
})
export class PgProfile {
    private changeDetector: ChangeDetectorRef = inject(ChangeDetectorRef);
    private dialogSrv: DialogSrv = inject(DialogSrv);
    private localeSrv: LocaleSrv = inject(LocaleSrv);
    private profileSrv: ProfileSrv = inject(ProfileSrv);
    private route: ActivatedRoute = inject(ActivatedRoute);
    private router: Router = inject(Router);
    private sessionSrv: SessionSrv = inject(SessionSrv);
    private translateSrv: TranslateService = inject(TranslateService);

    public errMsgObjsProfile: ErrMsgObj[] = [];
    public errMsgsPassword: string[] = [];
    public errMsgObjsPassword: ErrMsgObj[] = [];
    public errMsgsAccount: string[] = [];
    public errMsgObjsAccount: ErrMsgObj[] = [];
    public isLoadData = false;
    public profileConfigDto: ProfileConfigDto = this.route.snapshot.data["profileConfigDto"];
    public profileDto: ProfileDto = this.route.snapshot.data["profileDto"];

    // The current locale is loaded in the resolver.
    private loadedLangs: Record<string, boolean> = { [this.localeSrv.getLocale()]: true };

    private langChange$ = this.localeSrv.onLangChange.pipe(takeUntilDestroyed())
        .subscribe((event: LangChangeEvent) => {
            const title = `PgProfile().onLangChange(${event.lang})`;
            if (!this.loadedLangs[event.lang] && !!this.translateSrv.translations) {
                if (environment.logLevel > 0) {
                    console.log(`${title} translate.setTranslation(${event.lang}); trans:`, { ...event.translations });
                }
                this.loadedLangs[event.lang] = true;
                // Add translations from the main module to this module.
                this.translateSrv.setTranslation(event.lang, event.translations, true);
            }
            this.translateSrv.use(event.lang);
        });

    constructor() {
        if (environment.logLevel > 0) { console.log(`PgProfile(); locale.getLocale(): ${this.localeSrv.getLocale()}`); }
    }

    // ** Public API **

    // ** Section "Udate profile" FormGroup1 **

    public doUpdateProfile(obj: { modifyProfile: ModifyProfileDto, avatarFile: File | null | undefined }): void {
        if (!obj || !obj.modifyProfile) {
            return;
        }
        this.isLoadData = true;
        this.errMsgObjsProfile = [];
        this.profileSrv.modifyProfile(obj.modifyProfile, obj.avatarFile)
            .then((response: ProfileDto | HttpErrorResponse | undefined) => {
                if (response == null) {
                    this.errMsgObjsProfile = [{ msg: this.translateSrv.instant("pg-profile.error_editing_profile"), obj: null }];
                } else {
                    this.profileDto = response as ProfileDto;
                    this.sessionSrv.setUser(this.profileDto);
                    this.localeSrv.setLocale(this.profileDto.locale)
                        .finally(() => {
                            const title = this.translateSrv.instant("pg-profile.dialog_title_editing");
                            const message = this.translateSrv.instant("pg-profile.dialog_message_editing");
                            this.dialogSrv.openConfirmation(message, title, { btnNameAccept: "buttons.ok" }, { maxWidth: "40vw" });
                        });
                }
            })
            .catch((err: HttpErrorResponse) => {
                this.errMsgObjsProfile = HttpErrorUtil.mapErrMsgObjs(err.status, err.error);
            })
            .finally(() => {
                this.isLoadData = false;
                this.changeDetector.markForCheck();
            });
    }

    // ** Section "Set new password" FormGroup2 **

    public doUpdatePassword(newPasswordProfile: NewPasswordProfileDto): void { // UpdatePasswordDto
        if (!newPasswordProfile) {
            return;
        }
        this.isLoadData = true;
        this.errMsgObjsPassword = [];
        this.profileSrv.newPassword(newPasswordProfile)
            .then((response: ProfileDto | HttpErrorResponse | undefined) => {
                if (!response) {
                    const msg = this.translateSrv.instant("pg-profile.error_update_password", { nickname: this.profileDto.nickname });
                    this.errMsgObjsPassword = [{ msg, obj: null }];
                } else {
                    this.profileDto = response as ProfileDto;
                    const title = this.translateSrv.instant("pg-profile.dialog_title_password");
                    const message = this.translateSrv.instant("pg-profile.dialog_message_password");
                    this.dialogSrv.openConfirmation(message, title, { btnNameAccept: "buttons.ok" }, { maxWidth: "40vw" });
                }
            })
            .catch((err: HttpErrorResponse) => {
                this.errMsgObjsPassword = HttpErrorUtil.mapErrMsgObjs(err.status, err.error);
            })
            .finally(() => {
                this.isLoadData = false;
                this.changeDetector.markForCheck();
            });
    }

    // ** Section "Delete Account" **

    public doDeleteAccount(): void {
        this.isLoadData = true;
        this.errMsgObjsAccount = [];
        this.profileSrv.deleteCurrentProfile()
            .then((response: ProfileDto | HttpErrorResponse | undefined) => {
                const nickname = this.profileDto.nickname;
                if (!response) {
                    this.errMsgObjsAccount = [{ msg: this.translateSrv.instant("pg-profile.error_delete_account", { nickname }), obj: null }];
                } else {
                    // Closing the session.
                    this.sessionSrv.removeUserTokens();
                    this.sessionSrv.removeUser();
                    const title = this.translateSrv.instant("pg-profile.dialog_title_delete");
                    const message = this.translateSrv.instant("pg-profile.dialog_message_delete", { nickname });
                    this.dialogSrv.openConfirmation(message, title, { btnNameAccept: "buttons.ok" }, { maxWidth: "40vw" })
                        .finally(() => {
                            window.setTimeout(() => this.router.navigate([ROUTE_LOGIN]), 0);
                        })
                }
            })
            .catch((err: HttpErrorResponse) => {
                this.errMsgObjsAccount = HttpErrorUtil.mapErrMsgObjs(err.status, err.error);
            })
            .finally(() => {
                this.isLoadData = false;
                this.changeDetector.markForCheck();
            });
    }

    // ** Private API **
}
