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
import { UserService } from '../lib-user/user.service';
import { ModifyProfileDto, UpdatePasswordDto, UpdateProfileFileDto, UserDto } from '../lib-user/user-api.interface';
import { PanelProfileInfoComponent } from '../lib-profile/panel-profile-info/panel-profile-info.component';
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

  public userDto: UserDto;
  public isLoadData = false;
  public errMsgsProfile: string[] = [];
  public errMsgsPassword: string[] = [];
  public errMsgsAccount: string[] = [];
  
  constructor(
    private changeDetectorRef: ChangeDetectorRef,
    private route: ActivatedRoute,
    private router: Router,
    private translate: TranslateService,
    private dialogService: DialogService,
    private userService: UserService,
  ) {
    this.userDto = this.route.snapshot.data['userDto'];
  }
  
  // ** Public API **

  // ** Section "Udate profile" FormGroup1 **

  public doUpdateProfile(updateProfileFile: UpdateProfileFileDto): void {
    if (!updateProfileFile || (!updateProfileFile.id)) {
      return;
    }
    const modifyProfileDto: ModifyProfileDto = {
      nickname: updateProfileFile.nickname,
      email: updateProfileFile.email,
      password: updateProfileFile.password,
      descript: updateProfileFile.descript,
    };
    this.isLoadData = true;
    this.userService.modifyProfile(updateProfileFile.id, modifyProfileDto, updateProfileFile.avatarFile)
    //   .then(() => {
    //     Promise.resolve()
    //       .then(() => { // UserProfileDto
    //         this.goBack();
    //       });
    //   })
      .catch((error: HttpErrorResponse) => {
        this.errMsgsProfile = HttpErrorUtil.getMsgs(error);
      })
      .finally(() => {
        this.isLoadData = false;
        this.changeDetectorRef.markForCheck();
      });
  }

  // ** Section "Set new password" FormGroup2 **

  public doUpdatePassword(updatePasswordDto: UpdatePasswordDto): void {
    if (!updatePasswordDto) {
      return;
    }
    this.isLoadData = true;
    this.userService.new_password(updatePasswordDto)
      .then((response: UserDto | HttpErrorResponse | undefined) => {
        if (!response) {
          this.errMsgsPassword = [this.translate.instant('profile.error_update_password', { nickname: this.userDto.nickname })];
        } else {
          this.userDto = response as UserDto;
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
    this.userService.delete_user_current()
    .then((response: UserDto | HttpErrorResponse | undefined) => {
        if (!response) {
          this.errMsgsAccount = [this.translate.instant('profile.error_delete_account', { nickname: this.userDto.nickname })];
        } else {
          const title = this.translate.instant('profile.dialog_title_delete_account');
          const message = this.translate.instant('profile.dialog_message_delete_account');
          this.dialogService.openConfirmation(message, title, null, 'buttons.ok')
          .finally(() => {
            // Closing the session.
            this.userService.setUserDto(null);
            this.userService.setUserTokensDto(null);
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
