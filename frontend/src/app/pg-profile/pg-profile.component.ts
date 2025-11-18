import { CommonModule } from '@angular/common';
import { HttpErrorResponse } from '@angular/common/http';
import {
    ChangeDetectionStrategy, ChangeDetectorRef, Component, inject, Renderer2, ViewEncapsulation
} from '@angular/core';
import { ActivatedRoute, Router } from '@angular/router';
import { TranslateService } from '@ngx-translate/core';

import { SpinnerComponent } from '../components/spinner/spinner.component';
import { ROUTE_LOGIN } from '../common/routes';
import { DialogService } from '../lib-dialog/dialog.service';
import { PanelProfileComponent } from '../lib-profile/panel-profile/panel-profile.component';
import { ProfileDto, ModifyProfileDto, NewPasswordProfileDto } from '../lib-profile/profile-api.interface';
import { ProfileConfigDto } from '../lib-profile/profile-config.interface';
import { ProfileService } from '../lib-profile/profile.service';
import { HttpErrorUtil } from '../utils/http-error.util';
import { InitializationService } from '../common/initialization.service';
import { LocaleService } from '../common/locale.service';

@Component({
    selector: 'app-pg-profile',
    standalone: true,
    imports: [CommonModule, PanelProfileComponent, SpinnerComponent],
    templateUrl: './pg-profile.component.html',
    styleUrl: './pg-profile.component.scss',
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush
})
export class PgProfileComponent {

    private changeDetector: ChangeDetectorRef = inject(ChangeDetectorRef);
    private renderer: Renderer2 = inject(Renderer2);
    private route: ActivatedRoute = inject(ActivatedRoute);
    private router: Router = inject(Router);

    // private alertService: AlertService = inject(AlertService);
    private dialogService: DialogService = inject(DialogService);
    private localeService: LocaleService = inject(LocaleService);
    private initializationService: InitializationService = inject(InitializationService);
    private profileService: ProfileService = inject(ProfileService);
    // private streamService: StreamService = inject(StreamService);
    private translate: TranslateService = inject(TranslateService);


    public profileDto: ProfileDto = this.route.snapshot.data['profileDto'];
    public isLoadData = false;
    public errMsgsProfile: string[] = [];
    public errMsgsPassword: string[] = [];
    public errMsgsAccount: string[] = [];
    public profileConfigDto: ProfileConfigDto = this.route.snapshot.data['profileConfigDto'];

    // ** Public API **

    // ** Section "Udate profile" FormGroup1 **

    public doUpdateProfile(obj: { modifyProfile: ModifyProfileDto, avatarFile: File | null | undefined }): void {
        if (!obj || !obj.modifyProfile) {
            return;
        }
        this.isLoadData = true;
        this.profileService.modifyProfile(obj.modifyProfile, obj.avatarFile)
            .then((response: ProfileDto | HttpErrorResponse | undefined) => {
                if (response == null) {
                    this.errMsgsProfile = [this.translate.instant('pg-profile.error_editing_profile')];
                } else {
                    this.profileDto = response as ProfileDto;
                    this.profileService.setProfileDto({ ...this.profileDto });
                    this.initializationService.setColorScheme(this.profileService.profileDto?.theme, this.renderer);
                    this.localeService.setLocale(this.profileDto.locale)
                        .finally(() => {
                            const title = this.translate.instant('pg-profile.dialog_title_editing');
                            const message = this.translate.instant('pg-profile.dialog_message_editing');
                            this.dialogService.openConfirmation(message, title, { btnNameAccept: 'buttons.ok' }, { maxWidth: '40vw' });
                        });
                }
            })
            .catch((error: HttpErrorResponse) => {
                this.errMsgsProfile = HttpErrorUtil.getMsgs(error);
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
        this.profileService.newPassword(newPasswordProfile)
            .then((response: ProfileDto | HttpErrorResponse | undefined) => {
                if (!response) {
                    this.errMsgsPassword = [this.translate.instant('pg-profile.error_update_password', { nickname: this.profileDto.nickname })];
                } else {
                    this.profileDto = response as ProfileDto;
                    this.profileService.setProfileDto({ ...this.profileDto });
                    const title = this.translate.instant('pg-profile.dialog_title_password');
                    const message = this.translate.instant('pg-profile.dialog_message_password');
                    this.dialogService.openConfirmation(message, title, { btnNameAccept: 'buttons.ok' }, { maxWidth: '40vw' });
                }
            })
            .catch((error: HttpErrorResponse) => {
                this.errMsgsPassword = HttpErrorUtil.getMsgs(error);
            })
            .finally(() => {
                this.isLoadData = false;
                this.changeDetector.markForCheck();
            });
    }

    // ** Section "Delete Account" **

    public doDeleteAccount(): void {
        this.isLoadData = true;
        this.profileService.deleteProfileCurrent()
            .then((response: ProfileDto | HttpErrorResponse | undefined) => {
                const nickname = this.profileDto.nickname;
                if (!response) {
                    this.errMsgsAccount = [this.translate.instant('pg-profile.error_delete_account', { nickname })];
                } else {
                    // Closing the session.
                    this.profileService.setProfileDto(null);
                    this.profileService.setUserTokensDto(null);
                    const title = this.translate.instant('pg-profile.dialog_title_delete');
                    const message = this.translate.instant('pg-profile.dialog_message_delete', { nickname });
                    this.dialogService.openConfirmation(message, title, { btnNameAccept: 'buttons.ok' }, { maxWidth: '40vw' })
                        .finally(() => {
                            window.setTimeout(() => this.router.navigate([ROUTE_LOGIN]), 0);
                        })
                }
            })
            .catch((error: HttpErrorResponse) => {
                this.errMsgsAccount = HttpErrorUtil.getMsgs(error);
            })
            .finally(() => {
                this.isLoadData = false;
                this.changeDetector.markForCheck();
            });
    }

    // ** Private API **

}
