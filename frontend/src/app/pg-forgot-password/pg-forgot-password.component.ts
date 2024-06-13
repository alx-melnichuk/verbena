import { ChangeDetectionStrategy, ChangeDetectorRef, Component, ViewEncapsulation } from '@angular/core';
import { CommonModule } from '@angular/common';
import { HttpErrorResponse } from '@angular/common/http';
import { TranslateModule, TranslateService } from '@ngx-translate/core';
import { Router } from '@angular/router';

import { StrParams } from '../common/str-params';
import { ROUTE_LOGIN } from '../common/routes';
import { DialogService } from '../lib-dialog/dialog.service';
import { ForgotPasswordComponent } from '../lib-forgot-password/forgot-password/forgot-password.component';
import { UserService } from '../lib-user/user.service';
import { HttpErrorUtil } from '../utils/http-error.util';

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

  constructor(
    private changeDetector: ChangeDetectorRef,
    private router: Router,
    private translate: TranslateService,
    private dialogService: DialogService,
    private userService: UserService
  ) {
  }
  
  // ** Public API **

  public doResend(params: StrParams): void {
    if (!params) {
      return;
    }
    const email: string = params['email'] || "";

    if (!email) {
      return;
    }

    this.isDisabledSubmit = true;
    this.errMsgList = [];
    this.userService.recovery(email)
      .then(() => {
        const appName = this.translate.instant('app.name');
        const title = this.translate.instant('forgot-password.dialog_title', { appName: appName });
        const message = this.translate.instant('forgot-password.dialog_message', { value: email });
        this.dialogService.openConfirmation(message, title, null, 'buttons.ok').then(() => {
          this.router.navigateByUrl(ROUTE_LOGIN, { replaceUrl: true });
        });
      })
      .catch((error: HttpErrorResponse) => {
        this.errMsgList = HttpErrorUtil.getMsgs(error);
      })
      .finally(() => {
        this.isDisabledSubmit = false;
        this.changeDetector.markForCheck();
      });
  }

  // ** Private API **
}
