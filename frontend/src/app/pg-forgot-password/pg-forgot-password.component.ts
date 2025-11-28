import { ChangeDetectionStrategy, ChangeDetectorRef, Component, inject, ViewEncapsulation } from '@angular/core';
import { CommonModule } from '@angular/common';
import { HttpErrorResponse } from '@angular/common/http';
import { TranslatePipe, TranslateService } from '@ngx-translate/core';
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
    imports: [CommonModule, TranslatePipe, PanelForgotPasswordComponent],
    templateUrl: './pg-forgot-password.component.html',
    styleUrl: './pg-forgot-password.component.scss',
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush
})
export class PgForgotPasswordComponent {
    private changeDetector: ChangeDetectorRef = inject(ChangeDetectorRef);
    private dialogService: DialogService = inject(DialogService);
    private router: Router = inject(Router);
    private profileService: ProfileService = inject(ProfileService);
    private translate: TranslateService = inject(TranslateService);

    public isDisabledSubmit = false;
    public errMsgs: string[] = [];

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
                    window.setTimeout(() => this.router.navigateByUrl(ROUTE_LOGIN, { replaceUrl: true }), 0);
                });
            })
            .catch((err: HttpErrorResponse) => {
                this.errMsgs = HttpErrorUtil.getMsg(err.error, `${err.status} ${err.statusText}`);
            })
            .finally(() => {
                this.isDisabledSubmit = false;
                this.changeDetector.markForCheck();
            });
    }

    // ** Private API **
}
