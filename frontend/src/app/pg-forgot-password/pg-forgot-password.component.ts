import { ChangeDetectionStrategy, ChangeDetectorRef, Component, ViewEncapsulation } from '@angular/core';
import { CommonModule } from '@angular/common';
import { HttpErrorResponse } from '@angular/common/http';
import { TranslateModule, TranslateService } from '@ngx-translate/core';
import { Router } from '@angular/router';

import { StrParams } from '../common/str-params';
import { ROUTE_LOGIN } from '../common/routes';
import { DialogService } from '../lib-dialog/dialog.service';
import { PanelForgotPasswordComponent } from '../lib-forgot-password/panel-forgot-password/panel-forgot-password.component';
import { ProfileService } from '../lib-profile/profile.service';
import { HttpErrorUtil } from '../utils/http-error.util';

@Component({
  selector: 'app-pg-forgot-password',
  standalone: true,
  imports: [CommonModule, TranslateModule, PanelForgotPasswordComponent],
  templateUrl: './pg-forgot-password.component.html',
  styleUrls: ['./pg-forgot-password.component.scss'],
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class PgForgotPasswordComponent {
  public isDisabledSubmit = false;
  public errMsgs: string[] = [];

  constructor(
    private changeDetector: ChangeDetectorRef,
    private router: Router,
    private translate: TranslateService,
    private dialogService: DialogService,
    private profileService: ProfileService
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
    this.errMsgs = [];
    this.profileService.recovery(email)
      .then(() => {
        const appName = this.translate.instant('app.name');
        const title = this.translate.instant('pg-forgot-password.dialog_title', { appName: appName });
        const message = this.translate.instant('pg-forgot-password.dialog_message', { value: email });
        this.dialogService.openConfirmation(message, title, { btnNameAccept: 'buttons.ok' }).then(() => {
          this.router.navigateByUrl(ROUTE_LOGIN, { replaceUrl: true });
        });
      })
      .catch((error: HttpErrorResponse) => {
        this.errMsgs = HttpErrorUtil.getMsgs(error);
      })
      .finally(() => {
        this.isDisabledSubmit = false;
        this.changeDetector.markForCheck();
      });
  }

  // ** Private API **
}
