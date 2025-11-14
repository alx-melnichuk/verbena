import { ChangeDetectionStrategy, ChangeDetectorRef, Component, inject, ViewEncapsulation } from '@angular/core';
import { CommonModule } from '@angular/common';
import { HttpErrorResponse } from '@angular/common/http';
import { Router } from '@angular/router';

import { StrParams } from '../common/str-params';
import { REDIRECT_AFTER_LOGIN } from '../common/routes';
import { PanelLoginComponent } from '../lib-login/panel-login/panel-login.component';
import { ProfileService } from '../lib-profile/profile.service';
import { HttpErrorUtil } from '../utils/http-error.util';

@Component({
    selector: 'app-pg-login',
    standalone: true,
    imports: [CommonModule, PanelLoginComponent],
    templateUrl: './pg-login.component.html',
    styleUrl: './pg-login.component.scss',
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class PgLoginComponent {
    public isLogin = true;
    public isDisabledSubmit = false;
    public errMsgs: string[] = [];

    private changeDetector: ChangeDetectorRef = inject(ChangeDetectorRef);
    private router: Router = inject(Router);
    private profileService: ProfileService = inject(ProfileService);

    constructor() {
    }

    // ** Public API **

    public doLogin(params: StrParams): void {
        if (!params) {
            return;
        }
        const nickname: string = params['nickname'] || "";
        const password: string = params['password'] || "";

        if (!nickname || !password) {
            return;
        }
        this.isDisabledSubmit = true;
        this.errMsgs = [];
        this.profileService.login(nickname, password)
            .then(() => {
                window.setTimeout(() => {
                    this.router.navigateByUrl(REDIRECT_AFTER_LOGIN);
                }, 0);
            })
            .catch((error) => {
                this.errMsgs = HttpErrorUtil.getMsgs(error as HttpErrorResponse);
                throw error;
            })
            .finally(() => {
                this.isDisabledSubmit = false;
                this.changeDetector.markForCheck();
            })
    }

    // ** Private API **
}
