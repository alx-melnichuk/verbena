import { ChangeDetectionStrategy, ChangeDetectorRef, Component, ViewEncapsulation, inject } from '@angular/core';
import { CommonModule } from '@angular/common';
import { SignupComponent } from '../components/signup/signup.component';
import { StrParams } from '../common/str-params';
import { HttpErrorResponse } from '@angular/common/http';
import { TranslateModule, TranslateService } from '@ngx-translate/core';
import { UserService } from '../entities/user/user.service';
import { DialogService } from '../lib-dialog/dialog.service';
import { Router } from '@angular/router';
import { ROUTE_LOGIN } from '../common/routes';
import { AppErrorUtil } from '../common/app-error';

@Component({
  selector: 'app-pg-signup',
  standalone: true,
  imports: [CommonModule, TranslateModule, SignupComponent],
  templateUrl: './pg-signup.component.html',
  styleUrls: ['./pg-signup.component.scss'],
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush,
  providers: [DialogService],
})
export class PgSignupComponent {
  public isLogin = true;
  public isDisabledSubmit = false;
  public errMsgList: string[] = [];

  private defaultError: string;

  constructor(
    private changeDetector: ChangeDetectorRef,
    private router: Router,
    private translate: TranslateService,
    private dialogService: DialogService,
    private userService: UserService
  ) {
    this.defaultError = this.translate.instant('error.server_api_call');
  }

  // ** Public API **

  public doSignup(params: StrParams): void {
    if (!params) {
      return;
    }
    const nickname: string | null = params['nickname'];
    const password: string | null = params['password'];
    const email: string | null = params['email'];
    if (!nickname || !password || !email) {
      return;
    }

    this.isDisabledSubmit = true;
    this.errMsgList = [];
    this.userService
      .registration(nickname, email, password)
      .then(() => {
        const appName = this.translate.instant('app_name');
        const title = this.translate.instant('login.registration_title', { app_name: appName });
        const message = this.translate.instant('login.registration_message', { value: email });
        this.dialogService.openConfirmation(message, title, null, 'buttons.ok').then(() => {
          this.router.navigateByUrl(ROUTE_LOGIN, { replaceUrl: true });
        });
      })
      .catch((error: HttpErrorResponse) => {
        this.errMsgList = AppErrorUtil.handleError(error, this.defaultError);
      })
      .finally(() => {
        this.changeDetector.markForCheck();
        this.isDisabledSubmit = false;
      });
  }

  // ** Private API **
}
