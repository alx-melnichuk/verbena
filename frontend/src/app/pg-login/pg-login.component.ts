import { ChangeDetectionStrategy, ChangeDetectorRef, Component, ViewEncapsulation } from '@angular/core';
import { CommonModule } from '@angular/common';
import { HttpErrorResponse } from '@angular/common/http';
import { TranslateModule, TranslateService } from '@ngx-translate/core';
import { Router } from '@angular/router';

import { StrParams } from '../common/str-params';
import { ROUTE_VIEW } from '../common/routes';
import { LoginComponent } from '../components/login/login.component';
import { UserService } from '../entities/user/user.service';
import { HttpErrorUtil } from '../utils/http-error.util';

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
    private changeDetector: ChangeDetectorRef,
    private router: Router,
    private translate: TranslateService,
    private userService: UserService
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
      await this.userService.login(nickname, password);
      await this.userService.getCurrentUser();
      this.router.navigateByUrl(ROUTE_VIEW);
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
