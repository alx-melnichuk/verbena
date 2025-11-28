import { CommonModule } from '@angular/common';
import {
    ChangeDetectionStrategy, Component, forwardRef, Input, OnChanges, SimpleChanges, ViewChild, ViewEncapsulation
} from '@angular/core';
import {
    AbstractControl, ControlValueAccessor, FormControl, FormGroup, NG_VALIDATORS, NG_VALUE_ACCESSOR, ReactiveFormsModule,
    ValidationErrors, Validator
} from '@angular/forms';
import { MatFormFieldModule, SubscriptSizing } from '@angular/material/form-field';
import { MatInput, MatInputModule } from '@angular/material/input';
import { TranslatePipe } from '@ngx-translate/core';

import { ValidatorUtils } from 'src/app/utils/validator.utils';

export const INPUT = 'input';
export const CUSTOM_ERROR = 'customError';
export const TYPE_DEFAULT = 'text';
// https://stackoverflow.com/questions/386294/what-is-the-maximum-length-of-a-valid-email-address
// What is the maximum length of a valid email address? 
// Answer: An email address must not exceed 254 characters.
export const EMAIL_MAX_LENGTH = 254;
export const EMAIL_MIN_LENGTH = 5;
export const NICKNAME_MIN_LENGTH = 3;
export const NICKNAME_MAX_LENGTH = 64;
export const NICKNAME_PATTERN = '^[a-zA-Z]+[\\w]+$';

@Component({
    selector: 'app-field-input',
    exportAs: 'appFieldInput',
    standalone: true,
    imports: [CommonModule, ReactiveFormsModule, MatInputModule, MatFormFieldModule, TranslatePipe],
    templateUrl: './field-input.component.html',
    styleUrl: './field-input.component.scss',
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush,
    providers: [
        { provide: NG_VALUE_ACCESSOR, useExisting: forwardRef(() => FieldInputComponent), multi: true },
        { provide: NG_VALIDATORS, useExisting: forwardRef(() => FieldInputComponent), multi: true },
    ],
})
export class FieldInputComponent implements OnChanges, ControlValueAccessor, Validator {
    @Input()
    public gist: string = INPUT;
    @Input()
    public errorMsg: string | null | undefined;
    @Input()
    public hint: string | null | undefined;
    @Input()
    public isDisabled: boolean | null | undefined;
    @Input()
    public isReadOnly: boolean | null | undefined;
    @Input()
    public isRequired: boolean | null | undefined;
    @Input()
    public isSpellcheck: boolean | null | undefined;
    @Input()
    public label: string | null | undefined;
    @Input()
    public maxLen: number | null | undefined;
    @Input()
    public minLen: number | null | undefined;
    @Input()
    public pattern: string | null | undefined;
    @Input()
    public subscriptSizing: SubscriptSizing = 'fixed';
    @Input()
    public type: string | null | undefined = TYPE_DEFAULT;

    @ViewChild(MatInput, { static: false })
    public matInput: MatInput | null = null;

    public formControl: FormControl = new FormControl({ value: null, disabled: false }, []);
    public formGroup: FormGroup = new FormGroup({ input: this.formControl });

    ngOnChanges(changes: SimpleChanges): void {
        if (!!changes['isRequired'] || !!changes['minLen'] || !!changes['maxLen'] || !!changes['pattern'] || !!changes['type']) {
            this.prepareFormGroup(
                this.isRequired || null, this.maxLen || null, this.minLen || null, this.pattern || null, this.type || null);
        }
        if (!!changes['isDisabled']) {
            this.setDisabledState(!!this.isDisabled);
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
        return ValidatorUtils.getErrorMsg(errors, this.gist || INPUT);
    }

    public getFormControl(): FormControl {
        return this.formControl;
    }
    public markAsTouched(opts?: { onlySelf?: boolean; emitEvent?: boolean; }): void {
        this.formControl.markAsTouched(opts);
    }
    public markAllAsTouched(opts?: { emitEvent?: boolean; }): void {
        this.formControl.markAllAsTouched(opts);
    }
    public markAsUntouched(opts?: { onlySelf?: boolean; emitEvent?: boolean; }): void {
        this.formControl.markAsUntouched(opts);
    }
    public markAsDirty(opts?: { onlySelf?: boolean; emitEvent?: boolean; }): void {
        this.formControl.markAsDirty(opts);
    }
    public markAsPristine(opts?: { onlySelf?: boolean; emitEvent?: boolean; }): void {
        this.formControl.markAsPristine(opts);
    }
    public markAsPending(opts?: { onlySelf?: boolean; emitEvent?: boolean; }): void {
        this.formControl.markAsPending(opts);
    }

    // ** Private API **

    private errorMsgValidator = (control: AbstractControl): ValidationErrors | null => {
        return !!control && !!this.errorMsg ? { [CUSTOM_ERROR]: true } : null;
    };
    private prepareFormGroup(
        isRequired: boolean | null, maxLen: number | null, minLen: number | null, pattern: string | null, typeVal: string | null
    ): void {
        this.formControl.clearValidators();
        const paramsObj = {
            ...(!!isRequired ? { "required": true } : {}),
            ...(minLen != null && minLen > 0 ? { "minLength": minLen } : {}),
            ...(maxLen != null && maxLen > 0 ? { "maxLength": maxLen } : {}),
            ...(pattern != null && pattern.length > 0 ? { "pattern": pattern } : {}),
            ...(typeVal != null && typeVal == 'email' ? { 'email': true } : {})

        };
        this.formControl.setValidators([...ValidatorUtils.prepare(paramsObj), this.errorMsgValidator]);
        this.formControl.updateValueAndValidity();
    }
}
