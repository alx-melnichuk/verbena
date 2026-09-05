import { CommonModule } from "@angular/common";
import { ChangeDetectionStrategy, ChangeDetectorRef, Component, EventEmitter, HostBinding, HostListener, inject, Input, OnChanges, Output, SimpleChanges, ViewEncapsulation } from "@angular/core";
import { FormControl, FormGroup, ReactiveFormsModule } from "@angular/forms";
import { MatButtonModule } from "@angular/material/button";
import { MatFormFieldModule } from "@angular/material/form-field";
import { MatInputModule } from "@angular/material/input";
import { RouterLink } from "@angular/router";
import { TranslatePipe } from "@ngx-translate/core";
import { CN_NICKNAME, CN_EMAIL, CN_PASSWORD } from "../../common/fields-consts";
import { ROUTE_LOGIN } from "../../common/routes";
import { FieldInput } from "../../components/field-input/field-input";
import { FieldPassword } from "../../components/field-password/field-password";
import { UniquenessCheck } from "../../components/uniqueness-check/uniqueness-check";
import { UserSrv } from "../../lib-user/user-srv";
import { UniquenessDto } from "../../lib-user/user-dto";
import { ErrMsgObj } from "../../utils/http-error.util";

export const SG_DEBOUNCE_DELAY = 900;

@Component({
    selector: "app-panel-signup",
    exportAs: "appPanelSignup",
    standalone: true,
    imports: [CommonModule, RouterLink, ReactiveFormsModule, MatButtonModule, MatFormFieldModule, MatInputModule, TranslatePipe,
        FieldInput, FieldPassword, UniquenessCheck],
    templateUrl: "./panel-signup.html",
    styleUrl: "./panel-signup.scss",
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class PanelSignup implements OnChanges {
    @Input()
    public errMsgObjs: ErrMsgObj[] = [];
    @Input()
    public isDisabled: boolean | null | undefined;
    @Input()
    public checkUniqueFn: ((val: Record<string, string>) => Promise<boolean>) | null | undefined;

    @Output()
    readonly signup: EventEmitter<Record<string, string>> = new EventEmitter();

    private changeDetector: ChangeDetectorRef = inject(ChangeDetectorRef);
    private userSrv: UserSrv = inject(UserSrv);

    @HostBinding("class.global-scroll")
    public get isGlobalScroll(): boolean { return true; }

    public linkLogin = ROUTE_LOGIN;
    public debounceDelay: number = SG_DEBOUNCE_DELAY;

    public controls = {
        nickname: new FormControl<string | null>(null, []),
        email: new FormControl<string | null>(null, []),
        password: new FormControl<string | null>(null, []),
    };
    public formGroup: FormGroup = new FormGroup(this.controls);

    public cn_nickname = CN_NICKNAME;
    public cn_email = CN_EMAIL;
    public cn_password = CN_PASSWORD;

    @HostListener("document:keypress", ["$event"])
    public keyEvent(event: KeyboardEvent): void {
        if (event.code === "Enter") {
            this.doSignup();
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

    public doSignup(): void {
        if (this.formGroup.invalid || !!this.isDisabled) {
            return;
        }
        const nickname = this.controls.nickname.value || "";
        const password = this.controls.password.value || "";
        const email = this.controls.email.value || "";
        this.signup.emit({ nickname, email, password });
    }

    public updateErrMsgObjs(errMsgObjs: ErrMsgObj[] = []): void {
        this.errMsgObjs = errMsgObjs;
    }

    public checkUniqueNickname = (nickname: string | null | undefined): Promise<boolean> => {
        if (!nickname) {
            return Promise.resolve(true);
        }
        return this.userSrv.uniqueness(nickname, "").then((response) => response == null || (response as UniquenessDto).uniqueness);
    }

    public checkUniqueEmail = (email: string | null | undefined): Promise<boolean> => {
        if (!email) {
            return Promise.resolve(true);
        }
        return this.userSrv.uniqueness("", email).then((response) => response == null || (response as UniquenessDto).uniqueness);
    }
}
