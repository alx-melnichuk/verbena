import { ChangeDetectionStrategy, ChangeDetectorRef, Component, ViewEncapsulation } from '@angular/core';
import { CommonModule } from '@angular/common';
import { HttpErrorResponse } from '@angular/common/http';
import { TranslatePipe, TranslateService } from '@ngx-translate/core';
import { Router } from '@angular/router';

import { StrParams } from '../common/str-params';
import { ROUTE_LOGIN } from '../common/routes';
import { DialogService } from '../lib-dialog/dialog.service';
import { ProfileService } from '../lib-profile/profile.service';
import { PanelSignupComponent } from '../lib-signup/panel-signup/panel-signup.component';
import { HttpErrorUtil } from '../utils/http-error.util';

@Component({
    selector: 'app-pg-signup',
    standalone: true,
    imports: [CommonModule, TranslatePipe, PanelSignupComponent],
    templateUrl: './pg-signup.component.html',
    styleUrl: './pg-signup.component.scss',
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class PgSignupComponent {
    public isDisabledSubmit = false;
    public errMsgs: string[] = [];

    constructor(
        private changeDetector: ChangeDetectorRef,
        private router: Router,
        private translate: TranslateService,
        private dialogService: DialogService,
        private profileService: ProfileService,
    ) {
    }

    // ** Public API **

    public doSignup(params: StrParams): void {
        if (!params) {
            return;
        }
        const nickname: string = params['nickname'] || "";
        const password: string = params['password'] || "";
        const email: string = params['email'] || "";

        if (!nickname || !password || !email) {
            return;
        }

        this.isDisabledSubmit = true;
        this.errMsgs = [];
        this.profileService.registration(nickname, email, password)
            .then(() => {
                const appName = this.translate.instant('app.name');
                const title = this.translate.instant('pg-signup.dialog_title', { appName: appName });
                const message = this.translate.instant('pg-signup.dialog_message', { value: email });
                this.dialogService.openConfirmation(message, title, { btnNameAccept: 'buttons.ok' }).then(() => {
                    window.setTimeout(() => this.router.navigateByUrl(ROUTE_LOGIN, { replaceUrl: true }), 0);
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
