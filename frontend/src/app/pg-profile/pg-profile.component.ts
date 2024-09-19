import {
  ChangeDetectionStrategy, ChangeDetectorRef, Component, ViewEncapsulation
} from '@angular/core';
import { CommonModule } from '@angular/common';
import { HttpErrorResponse } from '@angular/common/http';
import { ActivatedRoute, Router } from '@angular/router';
import { TranslateService } from '@ngx-translate/core';

import { SpinnerComponent } from '../components/spinner/spinner.component';
import { ROUTE_LOGIN } from '../common/routes';
import { DialogService } from '../lib-dialog/dialog.service';
import { PanelProfileInfoComponent } from '../lib-profile/panel-profile-info/panel-profile-info.component';
import { ProfileDto, ModifyProfileDto, NewPasswordProfileDto } from '../lib-profile/profile-api.interface';
import { ProfileConfigDto } from '../lib-profile/profile-config.interface';
import { ProfileService } from '../lib-profile/profile.service';
import { HttpErrorUtil } from '../utils/http-error.util';

@Component({
  selector: 'app-pg-profile',
  standalone: true,
  imports: [CommonModule, PanelProfileInfoComponent, SpinnerComponent],
  templateUrl: './pg-profile.component.html',
  styleUrls: ['./pg-profile.component.scss'],
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class PgProfileComponent {

  public profileDto: ProfileDto;
  public isLoadData = false;
  public errMsgsProfile: string[] = [];
  public errMsgsPassword: string[] = [];
  public errMsgsAccount: string[] = [];
  public profileConfigDto: ProfileConfigDto;

  constructor(
    private changeDetectorRef: ChangeDetectorRef,
    private route: ActivatedRoute,
    private router: Router,
    private translate: TranslateService,
    private dialogService: DialogService,
    private profileService: ProfileService,
  ) {
    this.profileDto = this.route.snapshot.data['profileDto'];
    this.profileConfigDto = this.route.snapshot.data['profileConfigDto'];
  }
  
  // ** Public API **

  // ** Section "Udate profile" FormGroup1 **

  public doUpdateProfile(obj: { modifyProfile: ModifyProfileDto, avatarFile: File | null | undefined }): void {
    if (!obj || !obj.modifyProfile) {
      return;
    }
    this.isLoadData = true;
    this.profileService.modifyProfile(obj.modifyProfile, obj.avatarFile)
      .then((response: ProfileDto | HttpErrorResponse | undefined) => {
        if (!response) {
          this.errMsgsProfile = [this.translate.instant('profile.error_editing_profile')];
        } else {
          this.profileDto = response as ProfileDto;
          this.profileService.setProfileDto({...this.profileDto});
          const title = this.translate.instant('profile.dialog_title_editing');
          const message = this.translate.instant('profile.dialog_message_editing');
          this.dialogService.openConfirmation(message, title, null, 'buttons.ok');
        }          
       })
      .catch((error: HttpErrorResponse) => {
        this.errMsgsProfile = HttpErrorUtil.getMsgs(error);
      })
      .finally(() => {
        this.isLoadData = false;
        this.changeDetectorRef.markForCheck();
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
          this.errMsgsPassword = [this.translate.instant('profile.error_update_password', { nickname: this.profileDto.nickname })];
        } else {
          this.profileDto = response as ProfileDto;
          this.profileService.setProfileDto({...this.profileDto});
          const title = this.translate.instant('profile.dialog_title_password');
          const message = this.translate.instant('profile.dialog_message_password');
          this.dialogService.openConfirmation(message, title, null, 'buttons.ok');
        }
      })
      .catch((error: HttpErrorResponse) => {
        this.errMsgsPassword = HttpErrorUtil.getMsgs(error);
      })
      .finally(() => {
        this.isLoadData = false;
        this.changeDetectorRef.markForCheck();
      });
  }

  // ** Section "Delete Account" **

  public doDeleteAccount(): void {
    this.isLoadData = true;
    this.profileService.deleteProfileCurrent()
    .then((response: ProfileDto | HttpErrorResponse | undefined) => {
        const nickname = this.profileDto.nickname;
        if (!response) {
          this.errMsgsAccount = [this.translate.instant('profile.error_delete_account', { nickname })];
        } else {
          const title = this.translate.instant('profile.dialog_title_delete');
          const message = this.translate.instant('profile.dialog_message_delete', { nickname });
          this.dialogService.openConfirmation(message, title, null, 'buttons.ok')
          .finally(() => {
            // Closing the session.
            this.profileService.setProfileDto(null);
            this.profileService.setProfileTokensDto(null);
            this.router.navigate([ROUTE_LOGIN]);
          })
        }
      })
      .catch((error: HttpErrorResponse) => {
        this.errMsgsAccount = HttpErrorUtil.getMsgs(error);
      })
      .finally(() => {
        this.isLoadData = false;
        this.changeDetectorRef.markForCheck();
      });
  }

  // ** Private API **

}
