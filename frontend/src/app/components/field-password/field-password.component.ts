import {
    ChangeDetectionStrategy, Component, Input, OnChanges, SimpleChanges, ViewChild, ViewEncapsulation, forwardRef,
} from '@angular/core';
import { CommonModule } from '@angular/common';
import {
    AbstractControl, ControlValueAccessor, FormControl, FormGroup, NG_VALIDATORS, NG_VALUE_ACCESSOR, ReactiveFormsModule,
    ValidationErrors, Validator, ValidatorFn,
} from '@angular/forms';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatInput, MatInputModule } from '@angular/material/input';
import { TranslatePipe } from '@ngx-translate/core';

import { ValidatorUtils } from 'src/app/utils/validator.utils';

export const PASSWORD = "password";
export const PASSWORD_MIN_LENGTH = 6;
export const PASSWORD_MAX_LENGTH = 64;
export const PASSWORD_PATTERN = '^(?=.*[a-z])(?=.*[A-Z])(?=.*\\d)[A-Za-z\\d\\W_]{6,}$';
export const CUSTOM_ERROR = 'customError';

@Component({
    selector: 'app-field-password',
    exportAs: 'appFieldPassword',
    standalone: true,
    imports: [CommonModule, ReactiveFormsModule, MatInputModule, MatFormFieldModule, TranslatePipe],
    templateUrl: './field-password.component.html',
    styleUrl: './field-password.component.scss',
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush,
    providers: [
        { provide: NG_VALUE_ACCESSOR, useExisting: forwardRef(() => FieldPasswordComponent), multi: true },
        { provide: NG_VALIDATORS, useExisting: forwardRef(() => FieldPasswordComponent), multi: true },
    ],
})
export class FieldPasswordComponent implements OnChanges, ControlValueAccessor, Validator {
    @Input()
    public gist: string = PASSWORD;
    @Input()
    public errorMsg: string | null | undefined;
    @Input()
    public hint: string = '';
    @Input()
    public isDisabled: boolean = false;
    @Input()
    public isReadOnly: boolean = false;
    @Input()
    public isRequired: boolean = false;
    @Input()
    public isSpellcheck: boolean = false;
    @Input()
    public label: string = 'field-password.label';
    @Input()
    public maxLen: number = PASSWORD_MAX_LENGTH;
    @Input()
    public minLen: number = PASSWORD_MIN_LENGTH;
    @Input()
    public pattern: string = PASSWORD_PATTERN;
    @Input()
    public type: string = "text";

    @ViewChild(MatInput, { static: false })
    public matInput: MatInput | null = null;

    public formControl: FormControl = new FormControl({ value: null, disabled: false }, []);
    public formGroup: FormGroup = new FormGroup({ password: this.formControl });
    public isShowPassword = false;
    public errMessage: string = '';

    constructor() { }

    ngOnChanges(changes: SimpleChanges): void {
        if (!!changes['isRequired'] || !!changes['minLen'] || !!changes['maxLen'] || !!changes['pattern'] || !!changes['type']) {
            this.prepareFormGroup();
        }
        if (!!changes['isDisabled']) {
            this.setDisabledState(this.isDisabled);
        }
        if (!!changes['errorMsg']) {
            this.formControl.updateValueAndValidity();
            this.onChange(this.formControl.value);
        }
    }

    // ** ControlValueAccessor - start **

    public onChange: (val: string) => void = () => { };
    public onTouched: () => void = () => { };

    public writeValue(value: any): void {
        this.formControl.setValue(value, { emitEvent: true });
    }

    public registerOnChange(fn: any): void {
        this.onChange = fn;
    }

    public registerOnTouched(fn: any): void {
        this.onTouched = fn;
    }

    public setDisabledState(isDisabled: boolean): void {
        if (isDisabled != this.formGroup.disabled) {
            if (isDisabled) {
                this.isShowPassword = false;
                this.formGroup.disable();
            } else {
                this.formGroup.enable();
            }
        }
    }

    // ** ControlValueAccessor - finish **

    // ** Validator - start **

    public validate(control: AbstractControl): ValidationErrors | null {
        return this.formControl.errors;
    }

    // ** Validator - finish **

    // ** Public API **

    public focus(): void {
        this.matInput?.focus();
    }

    public getErrorMsg(errors: ValidationErrors | null): string {
        return ValidatorUtils.getErrorMsg(errors, this.gist || PASSWORD);
    }

    public showPassword(isShowPassword: boolean): void {
        if (this.isShowPassword !== isShowPassword) {
            this.isShowPassword = isShowPassword;
        }
    }

    // ** Private API **

    private errorMsgValidator = (control: AbstractControl): ValidationErrors | null => {
        const result = !!control && !!this.errorMsg ? { [CUSTOM_ERROR]: true } : null;
        return result;
    };
    private prepareFormGroup(): void {
        this.formControl.clearValidators();
        const paramsObj = {
            ...(this.isRequired ? { "required": true } : {}),
            ...(this.minLen > 0 ? { "minLength": this.minLen } : {}),
            ...(this.maxLen > 0 ? { "maxLength": this.maxLen } : {}),
            ...(this.pattern ? { "pattern": this.pattern } : {}),
            ...(this.type == "email" ? { "email": true } : {})
        };
        this.formControl.setValidators([...ValidatorUtils.prepare(paramsObj), this.errorMsgValidator]);
        this.formControl.updateValueAndValidity();
    }
}
