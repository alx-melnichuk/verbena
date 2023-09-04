import { ChangeDetectionStrategy, ChangeDetectorRef, Component, ViewEncapsulation } from '@angular/core';
import { CommonModule } from '@angular/common';
import { SignupComponent } from '../components/signup/signup.component';
import { StrParams } from '../common/str-params';
import { HttpErrorResponse } from '@angular/common/http';
import { TranslateModule, TranslateService } from '@ngx-translate/core';
import { UserService } from '../entities/user/user.service';

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
    private changeDetector: ChangeDetectorRef,
    // private router: Router,
    private translate: TranslateService, // private dialogService: DialogService,
    private userService: UserService
  ) {
    // this.isLogin = (window.location.pathname === ROUTE_LOGIN);
  }

  // ** Public API **

  public doSignup(params: StrParams): void {
    // this.signup.emit({ nickname, email, password });
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
        console.log(`PgSignup.doSignup() registration(); Ok`); // #
        // const title = this.translate.instant('login.registration_title');
        // const message = this.translate.instant('login.registration_message', { value: email });
        // this.dialogService.openConfirmation(message, title, null, 'buttons.ok')
        //   .then((result) => {
        //     this.router.navigateByUrl(ROUTE_LOGIN, { replaceUrl: true });
        //   });
      })
      .catch((error: HttpErrorResponse) => {
        this.errMsgList = this.handleError(error);
        this.changeDetector.markForCheck();
        console.log(`PgSignup.doSignup() registration(); err:`, error); // #
      })
      .finally(() => (this.isDisabledSubmit = false));
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
