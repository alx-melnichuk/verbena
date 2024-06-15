import {
  ChangeDetectionStrategy, ChangeDetectorRef, Component, ViewEncapsulation
} from '@angular/core';
import { CommonModule } from '@angular/common';
import { HttpErrorResponse } from '@angular/common/http';
import { ActivatedRoute, Router } from '@angular/router';

import { SpinnerComponent } from '../components/spinner/spinner.component';

import { AlertService } from '../lib-dialog/alert.service';
import { UserService } from '../lib-user/user.service';
import { ModifyProfileDto, UpdateProfileFileDto, UserDto } from '../lib-user/user-api.interface';
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
  public isLoadDataProfile = false;
  
  private goBackToRoute: string = '/ind/about';
  
  constructor(
    private changeDetectorRef: ChangeDetectorRef,
    private route: ActivatedRoute,
    private router: Router,
    private userService: UserService,
    private alertService: AlertService,
  ) {
    this.userDto = this.route.snapshot.data['userDto'];
    console.log(`PgProfile() userDto=`, this.userDto); // #
  }
  
  // ** Public API **

  public doCancelProfile(): void {
    this.goBack();
  }
  
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
    this.isLoadDataProfile = true;
    this.userService.modifyProfile(updateProfileFile.id, modifyProfileDto, updateProfileFile.avatarFile)
      .then(() => {
        Promise.resolve()
          .then(() => { // UserProfileDto
            this.goBack();
          });
      })
      .catch((error: HttpErrorResponse) => {
        console.error(`error: `, error); // #
        const title = 'profile.error_editing_profile';
        this.alertService.showError(HttpErrorUtil.getMsgs(error)[0], title);
        throw error;
      })
      .finally(() => {
        this.isLoadDataProfile = false;
        this.changeDetectorRef.markForCheck();
      });
  }

  // ** Private API **

  private goBack(): Promise<boolean> {
    return this.router.navigateByUrl(this.goBackToRoute);
  }
}
