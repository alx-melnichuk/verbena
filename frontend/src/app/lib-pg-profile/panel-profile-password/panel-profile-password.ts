import { CommonModule } from "@angular/common";
import {
    ChangeDetectionStrategy, ChangeDetectorRef, Component, EventEmitter, inject, Input, OnChanges, Output, SimpleChanges, ViewEncapsulation
} from "@angular/core";
import { AbstractControl, FormControl, FormGroup, ReactiveFormsModule, ValidationErrors, ValidatorFn } from "@angular/forms";
import { MatButtonModule } from "@angular/material/button";
import { MatInputModule } from "@angular/material/input";
import { TranslatePipe, TranslateService } from "@ngx-translate/core";
import { CN_PASSWORD } from "../../common/fields-consts";
import { FieldPassword } from "../../components/field-password/field-password";
import { ErrMsgObj } from "../../utils/http-error.util";
import { NewPasswordProfileDto } from "../profile-dto";

@Component({
    selector: 'app-panel-profile-password',
    exportAs: "appPanelProfilePassword",
    standalone: true,
    imports: [CommonModule, ReactiveFormsModule, MatButtonModule, MatInputModule, TranslatePipe, FieldPassword],
    templateUrl: './panel-profile-password.html',
    styleUrl: './panel-profile-password.scss',
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class PanelProfilePassword implements OnChanges {
    private changeDetector: ChangeDetectorRef = inject(ChangeDetectorRef);
    private translateSrv: TranslateService = inject(TranslateService);

    @Input()
    public errMsgObjs: ErrMsgObj[] = [];
    @Input()
    public isDisabledSubmit: boolean = false;

    @Output()
    readonly updatePassword: EventEmitter<NewPasswordProfileDto> = new EventEmitter();

    public cn_password = CN_PASSWORD;

    public cntlsPassword = {
        password: new FormControl(null, []),
        new_password: new FormControl(null, []),
    };
    public formGroupPassword: FormGroup = new FormGroup(this.cntlsPassword);
    public isRequiredPassword: boolean = false;

    constructor() {
        this.formGroupPassword.setValidators(this.validatorsForPassword());
    }

    ngOnChanges(changes: SimpleChanges): void {
        // # Added reset data.
        if (!!changes["isDisabledSubmit"]) {
            if (this.isDisabledSubmit != this.formGroupPassword.disabled) {
                this.isDisabledSubmit ? this.formGroupPassword.disable() : this.formGroupPassword.enable();
                this.changeDetector.markForCheck();
            }
        }
    }

    // ** Public API **

    // ** Section: Set new password (formPassword) **

    public validatorsForPassword(): ValidatorFn {
        return (control: AbstractControl): ValidationErrors | null => {
            const formGroup = control as FormGroup;
            const cntlPassword = formGroup.get("password");
            const cntlNewPassword = formGroup.get("new_password");
            if (cntlPassword?.pristine) {
                return { pristine: "password" };
            }
            if (cntlPassword?.invalid) {
                return { invalid: "password" };
            }
            if (cntlNewPassword?.pristine) {
                return { pristine: "new_password" };
            }
            if (cntlNewPassword?.invalid) {
                return { invalid: "new_password" };
            }
            const passwordValue = cntlPassword?.value || "";
            const newPasswordValue = cntlNewPassword?.value || "";
            if (!!passwordValue && !!newPasswordValue && passwordValue == newPasswordValue) {
                return { new_password_equal_to_old_value: true };
            }
            return null;
        };
    }

    public statePassword(passwordValue: string | null) {
        if (this.isRequiredPassword !== !!passwordValue) {
            this.isRequiredPassword = !!passwordValue;
        }
    }

    public checkPassword(formGroup: FormGroup): void {
        if (formGroup.errors != null && formGroup.errors["new_password_equal_to_old_value"]) {
            const fieldName = this.translateSrv.instant("panel-profile-password.new_password");
            this.errMsgObjs.push({ msg: "417.field-password:equal_to_old_value", obj: { "field-password": fieldName } });
        }
    }

    public setNewPassword(formGroup: FormGroup): void {
        const cntlPassword = formGroup.get("password");
        const cntlNewPassword = formGroup.get("new_password");
        if (formGroup.pristine || formGroup.invalid || !cntlPassword || !cntlNewPassword) {
            return;
        }
        const newPasswordProfileDto: NewPasswordProfileDto = {
            password: cntlPassword.value,
            newPassword: cntlNewPassword.value
        };
        this.updatePassword.emit(newPasswordProfileDto);
    }

    public updateErrMsgObjs(errMsgObjs: ErrMsgObj[] = []): void {
        this.errMsgObjs = errMsgObjs;
    }

    public getErrorObj(errors: unknown | null, value: string | null | undefined): ValidationErrors {
        return { ...(errors as ValidationErrors), ...{ [`field-password`]: value } };
    }

    // ** Private API **

}
