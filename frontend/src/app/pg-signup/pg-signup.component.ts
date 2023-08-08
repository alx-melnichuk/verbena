import { ChangeDetectionStrategy, Component, ViewEncapsulation } from '@angular/core';
import { CommonModule } from '@angular/common';
import { SignupComponent } from '../components/signup/signup.component';
import { StrParams } from '../common/str-params';
import { HttpErrorResponse } from '@angular/common/http';
import { TranslateModule, TranslateService } from '@ngx-translate/core';

@Component({
  selector: 'app-pg-signup',
  standalone: true,
  imports: [CommonModule, TranslateModule, SignupComponent],
  templateUrl: './pg-signup.component.html',
  styleUrls: ['./pg-signup.component.scss'],
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class PgSignupComponent {
  public isLogin = true;
  public isDisabledSubmit = false;
  public errMsgList: string[] = [];

  constructor(
    // private changeDetector: ChangeDetectorRef,
    // private router: Router,
    private translate: TranslateService // private dialogService: DialogService, // private profileService: ProfileService,
  ) {
    // this.isLogin = (window.location.pathname === ROUTE_LOGIN);
  }

  // ** Public API **

  public doSignup(params: StrParams): void {
    /*if (!params) { return; }
    const nickname = (params.nickname as string);
    const password = (params.password as string);
    const email = (params.email as string);
    if (!nickname || !password || !email) { return; }

    this.isDisabledSubmit = true;
    this.errMsgList = [];
    this.profileService.registration(nickname, email, password)
      .then(() => {
        const title = this.translateService.instant('login.registration_title');
        const message = this.translateService.instant('login.registration_message', { value: email });
        this.dialogService.openConfirmation(message, title, null, 'buttons.ok')
          .then((result) => {
            this.router.navigateByUrl(ROUTE_LOGIN, { replaceUrl: true });
          });
      })
      .catch((error: HttpErrorResponse) => {
        this.errMsgList = this.handleError(error);
        this.changeDetector.markForCheck();
      })
      .finally(() =>
        this.isDisabledSubmit = false);*/
  }

  // ** Private API **

  private handleError(error: HttpErrorResponse): string[] {
    let result: string[] = [this.translate.instant('error.server_api_call')];
    if (!!error) {
      const errMessage = error.error?.message;
      if (!!errMessage) {
        result = Array.isArray(errMessage) ? errMessage : [errMessage];
      } else if (!!error.message) {
        result = [error.message];
      }
    }
    return result;
  }
}
