import { ChangeDetectionStrategy, ChangeDetectorRef, Component, ViewEncapsulation } from '@angular/core';
import { CommonModule } from '@angular/common';
import { HttpErrorResponse } from '@angular/common/http';
import { Router } from '@angular/router';

import { StrParams } from '../common/str-params';
import { REDIRECT_AFTER_LOGIN } from '../common/routes';
import { LoginComponent } from '../lib-login/login/login.component';
import { ProfileService } from '../lib-profile/profile.service';
import { HttpErrorUtil } from '../utils/http-error.util';

@Component({
  selector: 'app-pg-login',
  standalone: true,
  imports: [CommonModule, LoginComponent],
  templateUrl: './pg-login.component.html',
  styleUrls: ['./pg-login.component.scss'],
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class PgLoginComponent {
  public isLogin = true;
  public isDisabledSubmit = false;
  public errMsgList: string[] = [];

  constructor(
    private changeDetector: ChangeDetectorRef,
    private router: Router,
    private profileService: ProfileService,
  ) {
  }

  // ** Public API **

  public async doLogin(params: StrParams): Promise<void> {
    if (!params) {
      return;
    }
    const nickname: string = params['nickname'] || "";
    const password: string = params['password'] || "";

    if (!nickname || !password) {
      return;
    }

    this.isDisabledSubmit = true;
    this.errMsgList = [];
    try {
      await this.profileService.login(nickname, password);
      this.router.navigateByUrl(REDIRECT_AFTER_LOGIN);
    } catch (error) {
      if (error instanceof HttpErrorResponse) {
        this.errMsgList = HttpErrorUtil.getMsgs(error);
      } else {
        throw error;
      }
    } finally {
      this.isDisabledSubmit = false;
      this.changeDetector.markForCheck();
    }
  }

  // ** Private API **
}
