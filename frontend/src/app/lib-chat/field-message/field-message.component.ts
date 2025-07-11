import { CommonModule } from '@angular/common';
import {
    ChangeDetectionStrategy, Component, forwardRef, Input, OnChanges,
    SimpleChanges, ViewChild, ViewEncapsulation
} from '@angular/core';
import {
    AbstractControl, FormControl, FormGroup, NG_VALIDATORS, NG_VALUE_ACCESSOR, ReactiveFormsModule, ValidationErrors
} from '@angular/forms';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatInput, MatInputModule } from '@angular/material/input';
import { CdkTextareaAutosize } from '@angular/cdk/text-field';
import { TranslatePipe } from '@ngx-translate/core';

import { ValidatorUtils } from 'src/app/utils/validator.utils';

export const MESSAGE = "textarea";
export const CUSTOM_ERROR = 'customError';
export const MESSAGE_MAX_ROWS = 6;
export const MESSAGE_MIN_ROWS = 2;
export const MESSAGE_MAX_LENGTH = 2048; // 2*1024
export const MESSAGE_MIN_LENGTH = 2;

@Component({
    selector: 'app-field-message',
    exportAs: 'appFieldMessage',
    standalone: true,
    imports: [CommonModule, ReactiveFormsModule, MatInputModule, MatFormFieldModule, TranslatePipe, CdkTextareaAutosize],
    templateUrl: './field-message.component.html',
    styleUrl: './field-message.component.scss',
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush,
    providers: [
        { provide: NG_VALUE_ACCESSOR, useExisting: forwardRef(() => FieldMessageComponent), multi: true },
        { provide: NG_VALIDATORS, useExisting: forwardRef(() => FieldMessageComponent), multi: true },
    ],
})
export class FieldMessageComponent implements OnChanges {
    @Input()
    public gist: string = MESSAGE;
    @Input()
    public errorMsg: string | null | undefined;
    @Input()
    public hint: string | null | undefined;
    @Input()
    public isDisabled: boolean = false;
    @Input()
    public isReadOnly: boolean = false;
    @Input()
    public isRequired: boolean = false;
    @Input()
    public isSpellcheck: boolean = false;
    @Input()
    public label: string | null | undefined;
    @Input()
    public maxLen: number | null = null;
    @Input()
    public minLen: number | null = null;
    @Input()
    public maxRows: number | null = null;
    @Input()
    public minRows: number | null = null;

    @ViewChild(MatInput, { static: false })
    public matInput: MatInput | null = null;

    public formControl: FormControl = new FormControl({ value: null, disabled: false }, []);
    public formGroup: FormGroup = new FormGroup({ textarea: this.formControl });
    public errMessage: string = '';
    public maxLenVal: number = MESSAGE_MAX_LENGTH;
    public minLenVal: number = MESSAGE_MIN_LENGTH;
    public maxRowsVal: number = MESSAGE_MAX_ROWS;
    public minRowsVal: number = MESSAGE_MIN_ROWS;

    constructor() {
    }

    ngOnChanges(changes: SimpleChanges): void {
        if (!!changes['isRequired'] || !!changes['minLen'] || !!changes['maxLen']) {
            if (!!changes['maxLen']) {
                this.maxLenVal = (!!this.maxLen && this.maxLen > 0 ? this.maxLen : MESSAGE_MAX_LENGTH);
            }
            if (!!changes['minLen']) {
                this.minLenVal = (!!this.minLen && this.minLen > 0 ? this.minLen : MESSAGE_MIN_LENGTH);
            }
            this.prepareFormGroup(this.isRequired, this.maxLenVal, this.minLenVal);
        }
        if (!!changes['isDisabled']) {
            this.setDisabledState(this.isDisabled);
        }
        if (!!changes['errorMsg']) {
            this.formControl.updateValueAndValidity();
            this.onChange(this.formControl.value);
        }
        if (!!changes['maxRows']) {
            this.maxRowsVal = (this.maxRows != null && this.maxRows > 0 ? this.maxRows : MESSAGE_MAX_ROWS);
        }
        if (!!changes['minRows']) {
            this.minRowsVal = (this.minRows != null && this.minRows > 0 ? this.minRows : MESSAGE_MIN_ROWS);
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
        return ValidatorUtils.getErrorMsg(errors, this.gist || MESSAGE);
    }

    // ** Private API **

    private errorMsgValidator = (control: AbstractControl): ValidationErrors | null => {
        const result = !!control && !!this.errorMsg ? { [CUSTOM_ERROR]: true } : null;
        return result;
    };
    private prepareFormGroup(isRequired: boolean, maxLen: number, minLen: number): void {
        this.formControl.clearValidators();
        const paramsObj = {
            ...(isRequired ? { "required": true } : {}),
            ...(minLen > 0 ? { "minLength": minLen } : {}),
            ...(maxLen > 0 ? { "maxLength": maxLen } : {}),
        };
        this.formControl.setValidators([...ValidatorUtils.prepare(paramsObj), this.errorMsgValidator]);
        this.formControl.updateValueAndValidity();
    }
}
