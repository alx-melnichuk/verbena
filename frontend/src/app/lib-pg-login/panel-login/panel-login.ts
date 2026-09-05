import { CommonModule } from "@angular/common";
import {
    ChangeDetectionStrategy, ChangeDetectorRef, Component, EventEmitter, HostBinding, HostListener, inject, Input, OnChanges, Output,
    SimpleChanges, ViewEncapsulation
} from "@angular/core";
import { ReactiveFormsModule, FormControl, FormGroup } from "@angular/forms";
import { MatButtonModule } from "@angular/material/button";
import { MatFormFieldModule } from "@angular/material/form-field";
import { MatInputModule } from "@angular/material/input";
import { RouterLink } from "@angular/router";
import { TranslatePipe } from "@ngx-translate/core";
import { FieldInput } from "../../components/field-input/field-input";
import { FieldPassword } from "../../components/field-password/field-password";
import { CN_NICKNAME, CN_EMAIL, CN_PASSWORD } from "../../common/fields-consts";
import { ROUTE_SIGNUP, ROUTE_FORGOT_PASSWORD } from "../../common/routes";
import { ErrMsgObj } from "../../utils/http-error.util";

@Component({
    selector: "app-panel-login",
    exportAs: "appPanelLogin",
    standalone: true,
    imports: [CommonModule, RouterLink, ReactiveFormsModule, MatButtonModule, MatFormFieldModule, MatInputModule, TranslatePipe,
        FieldInput, FieldPassword,],
    templateUrl: "./panel-login.html",
    styleUrl: "./panel-login.scss",
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class PanelLogin implements OnChanges {
    @Input()
    public errMsgObjs: ErrMsgObj[] = [];
    @Input()
    public isDisabled: boolean | null | undefined;

    @Output()
    readonly login: EventEmitter<Record<string, (string | null)>> = new EventEmitter();

    private changeDetector: ChangeDetectorRef = inject(ChangeDetectorRef);

    @HostBinding("class.global-scroll")
    public get isGlobalScroll(): boolean { return true; }

    public linkSignup = ROUTE_SIGNUP;
    public linkForgotPassword = ROUTE_FORGOT_PASSWORD;

    public controls = {
        nickname: new FormControl<string | null>(null, []),
        email: new FormControl<string | null>(null, []),
        password: new FormControl<string | null>(null, []),
    };
    public formGroup: FormGroup = new FormGroup(this.controls);

    public isEmail: boolean = false;
    public cn_nickname = CN_NICKNAME;
    public cn_email = CN_EMAIL;
    public cn_password = CN_PASSWORD;

    @HostListener("document:keypress", ["$event"])
    public keyEvent(event: KeyboardEvent): void {
        if (event.code === "Enter") {
            this.doLogin();
        }
    }

    ngOnChanges(changes: SimpleChanges): void {
        if (!!changes["isDisabled"]) {
            if (!!this.isDisabled != this.formGroup.disabled) {
                !!this.isDisabled ? this.formGroup.disable() : this.formGroup.enable();
                this.changeDetector.markForCheck();
            }
        }
    }

    // ** Public API **

    public doLogin(): void {
        if (this.formGroup.invalid || !!this.isDisabled) {
            return;
        }
        const nickname = this.controls.nickname.value;
        const password = this.controls.password.value;
        this.login.emit({ nickname, password });
    }

    public updateErrMsgObjs(errMsgObjs: ErrMsgObj[] = []): void {
        this.errMsgObjs = errMsgObjs;
    }

    public changeType(target: any): void {
        this.isEmail = (target || { "value": "" }).value.indexOf("@") > -1;
    }

    public nicknameFocusout(): void {
        let value = this.controls.nickname.value || "";
        let clearedValue = value.trim();
        if (value != clearedValue) {
            this.controls.nickname.setValue(clearedValue);
        }
    }
}
