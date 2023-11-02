import { ChangeDetectionStrategy, ChangeDetectorRef, Component, ViewEncapsulation } from '@angular/core';
import { CommonModule } from '@angular/common';
import { HttpErrorResponse } from '@angular/common/http';
import { TranslateModule, TranslateService } from '@ngx-translate/core';
import { Router } from '@angular/router';

import { AppErrorUtil } from '../common/app-error';
import { ForgotPasswordComponent } from '../components/forgot-password/forgot-password.component';
import { StrParams } from '../common/str-params';
import { ROUTE_LOGIN } from '../common/routes';
import { UserService } from '../entities/user/user.service';
import { DialogService } from '../lib-dialog/dialog.service';

@Component({
  selector: 'app-pg-forgot-password',
  standalone: true,
  imports: [CommonModule, TranslateModule, ForgotPasswordComponent],
  templateUrl: './pg-forgot-password.component.html',
  styleUrls: ['./pg-forgot-password.component.scss'],
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class PgForgotPasswordComponent {

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

  public doResend(params: StrParams): void {
    if (!params) {
      return;
    }
    const email: string | null = params['email'];
    if (!email) {
      return;
    }

    this.isDisabledSubmit = true;
    this.errMsgList = [];
    this.userService.recovery(email)
      .then(() => {
        const appName = this.translate.instant('app_name');
        const title = this.translate.instant('forgot-password.dialog_title', { app_name: appName });
        const message = this.translate.instant('forgot-password.dialog_message', { value: email });
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
