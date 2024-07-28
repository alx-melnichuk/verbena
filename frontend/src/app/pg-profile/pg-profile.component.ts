import {
  ChangeDetectionStrategy, ChangeDetectorRef, Component, ViewEncapsulation
} from '@angular/core';
import { CommonModule } from '@angular/common';
import { HttpErrorResponse } from '@angular/common/http';
import { ActivatedRoute } from '@angular/router';
import { TranslateService } from '@ngx-translate/core';

import { SpinnerComponent } from '../components/spinner/spinner.component';

import { AlertService } from '../lib-dialog/alert.service';
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
  
  private goBackToRoute: string = '/ind/about';
  
  constructor(
    private changeDetectorRef: ChangeDetectorRef,
    private route: ActivatedRoute,
    private translate: TranslateService,
    private dialogService: DialogService,
    private userService: UserService,
    private alertService: AlertService,
  ) {
    this.userDto = this.route.snapshot.data['userDto'];
    console.log(`PgProfile() userDto=`, this.userDto); // #
  }
  
  // ** Public API **

  public doUpdateProfile(updateProfileFile: UpdateProfileFileDto): void {
    this.alertService.hide();
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
        console.error(`error: `, error); // #
        //  const title = 'profile.error_editing_profile';
        this.errMsgsProfile = HttpErrorUtil.getMsgs(error);
        // this.alertService.showError(this.errMsgsProfile[0], title);
        throw error;
      })
      .finally(() => {
        this.isLoadData = false;
        this.changeDetectorRef.markForCheck();
      });
  }


  public doUpdatePassword(updatePasswordDto: UpdatePasswordDto): void {
    this.alertService.hide();
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

  // ** Private API **

}
