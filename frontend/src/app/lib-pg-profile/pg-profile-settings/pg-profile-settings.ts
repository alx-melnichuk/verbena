import { CommonModule } from '@angular/common';
import { HttpErrorResponse } from '@angular/common/http';
import {
    ChangeDetectionStrategy, ChangeDetectorRef, Component, HostBinding, inject, OnDestroy, OnInit, ViewEncapsulation
} from '@angular/core';
import { ActivatedRoute } from '@angular/router';
import { TranslateService } from '@ngx-translate/core';
import { environment as env } from "../../../environments/environment";
import { LocaleSrv } from '../../common/locale-srv';
import { LOG_PG_PROFILE } from '../../common/routes';
import { HasUnsavedChanges, UnsavedDeActivateUtil } from '../../common/unsaved-de-activate-page-guard';
import { Spinner } from '../../components/spinner/spinner';
import { DialogSrv } from '../../lib-dialog/dialog-srv';
import { UserDto, SettingsDto } from '../../lib-user/user-dto';
import { ErrMsgObj, HttpErrorUtil } from '../../utils/http-error.util';
import { ProfileSrv } from '../profile-srv';
import { PanelProfileSettings } from '../panel-profile-settings/panel-profile-settings';

@Component({
    selector: 'app-pg-profile-settings',
    exportAs: "appPgProfileSettings",
    standalone: true,
    imports: [CommonModule, Spinner, PanelProfileSettings],
    templateUrl: './pg-profile-settings.html',
    styleUrl: './pg-profile-settings.scss',
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class PgProfileSettings implements OnInit, OnDestroy, HasUnsavedChanges {
    private changeDetector: ChangeDetectorRef = inject(ChangeDetectorRef);
    private route: ActivatedRoute = inject(ActivatedRoute);
    private dialogSrv: DialogSrv = inject(DialogSrv);
    private localeSrv: LocaleSrv = inject(LocaleSrv);
    private profileSrv: ProfileSrv = inject(ProfileSrv);
    private translateSrv: TranslateService = inject(TranslateService);

    public errMsgObjs: ErrMsgObj[] = [];
    private changeObj: Record<string, boolean> = {};
    private isChangeData: boolean = false;
    public isLoadData = false;
    public profileDto: UserDto;
    public settingsDto: SettingsDto;

    @HostBinding("class.global-scroll")
    public get classGlobalScrollVal(): boolean {
        return true;
    }

    constructor() {
        if (env.logLevel & LOG_PG_PROFILE) { console.info(`PgProfileSettings(); locale.getLocale(): ${this.localeSrv.getLocale()}`); }

        this.profileDto = this.route.snapshot.data["profileDto"];
        this.settingsDto = { ...this.profileDto.settings };
        this.profileDto.settings = undefined;
    }

    ngOnInit(): void {
        console.log(`PgProfileSettings.OnInit();`); // #
        // Set a confirmation string when attempting to leave a page with unsaved data.
        UnsavedDeActivateUtil.setConfirmText("pg-profile-settings.have_unsaved_you_want_to_leave");
    }

    ngOnDestroy() {
        console.log(`PgProfileSettings.OnDestroy();`); // #
        // Reset a confirmation string when attempting to leave a page with unsaved data.
        UnsavedDeActivateUtil.setConfirmText("");
    }

    // ** HasUnsavedChanges **
    public hasUnsavedChanges(): boolean {
        return this.isChangeData;
    }

    // ** Public API **

    public doChangeData(isChange: boolean): void {
        this.isChangeData = isChange;
        console.log(`doChangeData() isChange : ${this.isChangeData}`); // #
    }

    // ** Section "Set new Settings" **

    public doUpdateSettings(settingsDto: SettingsDto | null): void {
        this.isLoadData = true;
        this.errMsgObjs = [];

        this.profileSrv.modifySettings(settingsDto)
            .then((response: UserDto | HttpErrorResponse | undefined) => {
                if (response == null) {
                    const msg = this.translateSrv.instant("pg-profile-settings.error_save_settings", { nickname: this.profileDto.nickname });
                    this.errMsgObjs = [{ msg, obj: null }];
                } else {
                    this.profileDto = response as UserDto;
                    this.settingsDto = { ...this.profileDto.settings };
                    this.profileDto.settings = undefined;
                    this.isChangeData = false;

                    this.dialogSrv.openConfirmation("pg-profile-settings.dialog_message_settings"
                        , "pg-profile-settings.dialog_title_settings"
                        , { btnNameAccept: "buttons.ok" }, { maxWidth: "40vw" });
                }
            })
            .catch((err: HttpErrorResponse) => {
                this.errMsgObjs = HttpErrorUtil.mapErrMsgObjs(err.status, err.error);
            })
            .finally(() => {
                this.isLoadData = false;
                this.changeDetector.markForCheck();
            });
    }
}
