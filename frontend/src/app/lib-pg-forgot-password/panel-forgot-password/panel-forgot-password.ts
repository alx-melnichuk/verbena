import { CommonModule } from "@angular/common";
import {
    ChangeDetectionStrategy, Component, EventEmitter, HostBinding, HostListener, Input, OnChanges, Output, SimpleChanges, ViewEncapsulation
} from "@angular/core";
import { FormControl, FormGroup, ReactiveFormsModule } from "@angular/forms";
import { MatButtonModule } from "@angular/material/button";
import { MatFormFieldModule } from "@angular/material/form-field";
import { MatInputModule } from "@angular/material/input";
import { RouterLink } from "@angular/router";
import { TranslatePipe } from "@ngx-translate/core";
import { ROUTE_LOGIN } from "../../common/routes";
import { FieldInput } from "../../components/field-input/field-input";
import { CN_EMAIL } from "../../common/fields-consts";
import { ErrMsgObj } from "../../utils/http-error.util";

@Component({
    selector: "app-panel-forgot-password",
    exportAs: "appPanelForgotPassword",
    standalone: true,
    imports: [CommonModule, RouterLink, ReactiveFormsModule, MatButtonModule, MatFormFieldModule, MatInputModule, TranslatePipe,
        FieldInput],
    templateUrl: "./panel-forgot-password.html",
    styleUrl: "./panel-forgot-password.scss",
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class PanelForgotPassword implements OnChanges {
    @Input()
    public errMsgObjs: ErrMsgObj[] = [];
    @Input()
    public isDisabled: boolean | null | undefined;
    @Output()
    readonly resend: EventEmitter<Record<string, (string | null)>> = new EventEmitter();

    @HostBinding("class.global-scroll")
    public get isGlobalScroll(): boolean { return true; }

    public linkLogin = ROUTE_LOGIN;

    public controls = {
        email: new FormControl<string | null>(null, []),
    };
    public formGroup: FormGroup = new FormGroup(this.controls);

    public cn_email = CN_EMAIL;

    @HostListener("document:keypress", ["$event"])
    public keyEvent(event: KeyboardEvent): void {
        if (event.code === "Enter") {
            this.doResend();
        }
    }

    ngOnChanges(changes: SimpleChanges): void {
        if (!!changes["isDisabled"]) {
            if (!!this.isDisabled != this.formGroup.disabled) {
                !!this.isDisabled ? this.formGroup.disable() : this.formGroup.enable();
            }
        }
    }

    // ** Public API **

    public doResend(): void {
        if (this.formGroup.invalid || !!this.isDisabled) {
            return;
        }
        const email = this.controls.email.value;
        this.resend.emit({ email });
    }

    public updateErrMsgObjs(errMsgObjs: ErrMsgObj[] = []): void {
        this.errMsgObjs = errMsgObjs;
    }

}
