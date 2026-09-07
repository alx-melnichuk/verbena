import { CommonModule } from '@angular/common';
import { HttpErrorResponse } from '@angular/common/http';
import {
    ChangeDetectionStrategy, ChangeDetectorRef, Component, HostBinding, inject, OnDestroy, OnInit, ViewEncapsulation
} from '@angular/core';
import { ActivatedRoute, Router } from '@angular/router';
import { TranslateService } from '@ngx-translate/core';
import { environment as env } from "../../../environments/environment";
import { LocaleSrv } from '../../common/locale-srv';
import { LOG_PG_PROFILE, ROUTE_LOGIN } from '../../common/routes';
import { SessionSrv } from '../../common/session-srv';
import { HasUnsavedChanges, UnsavedDeActivateUtil } from '../../common/unsaved-de-activate-page-guard';
import { Spinner } from '../../components/spinner/spinner';
import { DialogSrv } from '../../lib-dialog/dialog-srv';
import { UserDto } from '../../lib-user/user-dto';
import { ErrMsgObj, HttpErrorUtil } from '../../utils/http-error.util';
import { PanelProfileInfo } from '../panel-profile-info/panel-profile-info';
import { PanelProfilePassword } from '../panel-profile-password/panel-profile-password';
import { PanelProfileRemove } from '../panel-profile-remove/panel-profile-remove';
import { ProfileConfigDto } from '../profile-config-dto';
import { ModifyProfileDto, NewPasswordProfileDto } from '../profile-dto';
import { ProfileSrv } from '../profile-srv';

@Component({
    selector: 'app-pg-profile-details',
    exportAs: "appPgProfileDetails",
    standalone: true,
    imports: [CommonModule, Spinner, PanelProfileInfo, PanelProfilePassword, PanelProfileRemove],
    templateUrl: './pg-profile-details.html',
    styleUrl: './pg-profile-details.scss',
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class PgProfileDetails implements OnInit, OnDestroy, HasUnsavedChanges {
    private changeDetector: ChangeDetectorRef = inject(ChangeDetectorRef);
    private route: ActivatedRoute = inject(ActivatedRoute);
    private router: Router = inject(Router);
    private dialogSrv: DialogSrv = inject(DialogSrv);
    private localeSrv: LocaleSrv = inject(LocaleSrv);
    private profileSrv: ProfileSrv = inject(ProfileSrv);
    private sessionSrv: SessionSrv = inject(SessionSrv);
    private translateSrv: TranslateService = inject(TranslateService);

    public errMsgObjsProfile: ErrMsgObj[] = [];
    public errMsgObjsPassword: ErrMsgObj[] = [];
    public errMsgObjsAccount: ErrMsgObj[] = [];
    public isChangeDataInfo: boolean = false;
    public isChangeDataPassword: boolean = false;
    public isLoadData = false;
    public isResetAccount = false;
    public profileConfigDto: ProfileConfigDto = this.route.snapshot.data["profileConfigDto"];
    public profileDto: UserDto;

    @HostBinding("class.global-scroll")
    public get classGlobalScrollVal(): boolean {
        return true;
    }

    constructor() {
        if (env.logLevel & LOG_PG_PROFILE) { console.info(`PgProfileDetails(); locale.getLocale(): ${this.localeSrv.getLocale()}`); }

        this.profileDto = this.route.snapshot.data["profileDto"];
        this.profileDto.settings = undefined;
    }

    ngOnInit(): void {
        // Set a confirmation string when attempting to leave a page with unsaved data.
        UnsavedDeActivateUtil.setConfirmText("pg-profile-details.have_unsaved_you_want_to_leave");
    }

    ngOnDestroy() {
        // Reset a confirmation string when attempting to leave a page with unsaved data.
        UnsavedDeActivateUtil.setConfirmText("");
    }

    // ** HasUnsavedChanges **
    public hasUnsavedChanges(): boolean {
        return this.isChangeDataInfo || this.isChangeDataPassword;
    }

    // ** Public API **

    public doChangeDataInfo(isChange: boolean): void {
        this.isChangeDataInfo = isChange;
        console.log(`doChangeDataInfo() isChangeInfo : ${this.isChangeDataInfo}`); // #
    }
    public doChangeDataPassword(isChange: boolean): void {
        this.isChangeDataPassword = isChange;
        console.log(`doChangeDataPassword() isChangePassword : ${this.isChangeDataPassword}`); // #
    }

    // ** Section "Udate profile" FormGroup1 **

    public doUpdateProfile(obj: { modifyProfile: ModifyProfileDto, avatarFile: File | null | undefined }): void {
        if (!obj || !obj.modifyProfile) {
            return;
        }
        this.isLoadData = true;
        this.errMsgObjsProfile = [];
        this.profileSrv.modifyProfile(obj.modifyProfile, obj.avatarFile)
            .then((response: UserDto | HttpErrorResponse | undefined) => {
                if (response == null) {
                    this.errMsgObjsProfile = [{ msg: this.translateSrv.instant("pg-profile-details.error_editing_profile"), obj: null }];
                } else {
                    this.profileDto = response as UserDto;
                    this.profileDto.settings = undefined;
                    this.isChangeDataInfo = false;

                    this.sessionSrv.setUser(this.profileDto);
                    this.localeSrv.setLocale(this.profileDto.locale)
                        .finally(() => {
                            this.dialogSrv.openConfirmation("pg-profile-details.dialog_message_editing"
                                , "pg-profile-details.dialog_title_editing"
                                , { btnNameAccept: "buttons.ok" }, { maxWidth: "40vw" });
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
        this.isResetAccount = false;
        this.profileSrv.newPassword(newPasswordProfile)
            .then((response: UserDto | HttpErrorResponse | undefined) => {
                if (!response) {
                    const msg = this.translateSrv.instant("pg-profile-details.error_update_password"
                        , { nickname: this.profileDto.nickname });
                    this.errMsgObjsPassword = [{ msg, obj: null }];
                } else {
                    this.profileDto = response as UserDto;
                    this.isResetAccount = true;
                    this.isChangeDataPassword = false;

                    this.dialogSrv.openConfirmation("pg-profile-details.dialog_message_password"
                        , "pg-profile-details.dialog_title_password"
                        , { btnNameAccept: "buttons.ok" }, { maxWidth: "40vw" });
                }
            })
            .catch((err: HttpErrorResponse) => {
                this.errMsgObjsPassword = HttpErrorUtil.mapErrMsgObjs(err.status, err.error || "error.server_api_call");
                throw err;
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
            .then((response: UserDto | HttpErrorResponse | undefined) => {
                const nickname = this.profileDto.nickname;
                if (!response) {
                    this.errMsgObjsAccount = [{ msg: this.translateSrv.instant("pg-profile-details.error_delete_account", { nickname }), obj: null }];
                } else {
                    // Closing the session.
                    this.sessionSrv.removeUserTokens();
                    this.sessionSrv.removeUser();
                    const message = this.translateSrv.instant("pg-profile-details.dialog_message_delete", { nickname });
                    this.dialogSrv.openConfirmation(message, "pg-profile-details.dialog_title_delete"
                        , { btnNameAccept: "buttons.ok" }, { maxWidth: "40vw" })
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
