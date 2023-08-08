import { ChangeDetectionStrategy, Component, ViewEncapsulation } from '@angular/core';
import { CommonModule } from '@angular/common';
import { LoginComponent } from '../components/login/login.component';
import { StrParams } from '../common/str-params';
import { HttpErrorResponse } from '@angular/common/http';
import { TranslateModule, TranslateService } from '@ngx-translate/core';

@Component({
  selector: 'app-pg-login',
  standalone: true,
  imports: [CommonModule, TranslateModule, LoginComponent],
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
    // private changeDetector: ChangeDetectorRef,
    // private router: Router,
    private translate: TranslateService // private dialogService: DialogService, // private profileService: ProfileService,
  ) {
    // this.isLogin = (window.location.pathname === ROUTE_LOGIN);
  }

  // ** Public API **

  public async doLogin(params: StrParams): Promise<void> {
    /*if (!params) { return; }
    const nickname = (params.nickname as string);
    const password = (params.password as string);
    if (!nickname || !password) { return; }

    this.isDisabledSubmit = true;
    this.errMsgList = [];
    try {
      await this.profileService.authentication(nickname, password);
      await this.profileService.getProfileSession();
      this.router.navigateByUrl(ROUTE_ROOT);
    } catch (error) {
      if (error instanceof HttpErrorResponse) {
        if (error.status === 403) {
          this.errMsgList = [this.translateService.instant('login.err_not_correct_password')];
        } else {
          this.errMsgList = this.handleError(error);
        }
      } else {
        throw error;
      }
    } finally {
      this.isDisabledSubmit = false;
      this.changeDetector.markForCheck();
    }*/
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
