import { CommonModule } from '@angular/common';
import {
    ChangeDetectionStrategy, ChangeDetectorRef, Component, EventEmitter, HostBinding, HostListener, inject, Input, OnChanges, Output,
    SimpleChanges, ViewEncapsulation
} from '@angular/core';
import { FormControl, FormGroup, ReactiveFormsModule } from '@angular/forms';
import { RouterLink } from '@angular/router';
import { MatButtonModule } from '@angular/material/button';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatInputModule } from '@angular/material/input';
import { TranslatePipe } from '@ngx-translate/core';

import { EMAIL_MAX_LENGTH, EMAIL_MIN_LENGTH, FieldInputComponent } from 'src/app/components/field-input/field-input.component';
import { ROUTE_LOGIN } from 'src/app/common/routes';
import { StrParams } from 'src/app/common/str-params';

@Component({
    selector: 'app-panel-forgot-password',
    exportAs: 'appPanelForgotPassword',
    standalone: true,
    imports: [CommonModule, RouterLink, ReactiveFormsModule, MatButtonModule, MatFormFieldModule, MatInputModule, TranslatePipe,
        FieldInputComponent],
    templateUrl: './panel-forgot-password.component.html',
    styleUrl: './panel-forgot-password.component.scss',
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush
})
export class PanelForgotPasswordComponent implements OnChanges {
    @Input()
    public isDisabledSubmit: boolean = false;
    @Input()
    public errMsgs: string[] = [];
    @Output()
    readonly resend: EventEmitter<StrParams> = new EventEmitter();

    @HostBinding('class.global-scroll')
    public get isGlobalScroll(): boolean { return true; }

    public linkLogin = ROUTE_LOGIN;

    public controls = {
        email: new FormControl<string | null>(null, []),
    };
    public formGroup: FormGroup = new FormGroup(this.controls);

    public emailMinLen: number = EMAIL_MIN_LENGTH;
    public emailMaxLen: number = EMAIL_MAX_LENGTH;

    private changeDetector: ChangeDetectorRef = inject(ChangeDetectorRef);

    @HostListener('document:keypress', ['$event'])
    public keyEvent(event: KeyboardEvent): void {
        if (event.code === 'Enter') {
            this.doResend();
        }
    }

    ngOnChanges(changes: SimpleChanges): void {
        if (!!changes['isDisabledSubmit']) {
            if (this.isDisabledSubmit != this.formGroup.disabled) {
                this.isDisabledSubmit ? this.formGroup.disable() : this.formGroup.enable();
                this.changeDetector.markForCheck();
            }
        }
    }

    // ** Public API **

    public doResend(): void {
        if (this.formGroup.invalid || this.isDisabledSubmit) {
            return;
        }
        const email = this.controls.email.value;
        this.resend.emit({ email });
    }

    public updateErrMsg(errMsgs: string[] = []): void {
        this.errMsgs = errMsgs;
    }
}
