import { CommonModule } from "@angular/common";
import {
    ChangeDetectionStrategy, ChangeDetectorRef, Component, EventEmitter, forwardRef, inject, Input, OnChanges, Output, SimpleChanges,
    ViewEncapsulation
} from "@angular/core";
import {
    ReactiveFormsModule, ControlValueAccessor, Validator, FormControl, FormGroup, AbstractControl, ValidationErrors,
    NG_VALIDATORS, NG_VALUE_ACCESSOR
} from "@angular/forms";
import { MatButtonModule } from "@angular/material/button";
import { MatFormFieldModule } from "@angular/material/form-field";
import { MatInputModule } from "@angular/material/input";
import { MatTooltipModule } from "@angular/material/tooltip";
import { TranslatePipe } from "@ngx-translate/core";
import { ValidatorUtils } from "../../utils/validator.utils";
import { FieldFileUpload } from "../field-file-upload/field-file-upload";
import { Image } from "../image/image";

export const CUSTOM_ERROR = "customError";
export const IMAGE = "image";

@Component({
    selector: "app-field-image",
    exportAs: "appFieldImage",
    standalone: true,
    imports: [CommonModule, ReactiveFormsModule, MatButtonModule, MatFormFieldModule,
        MatInputModule, MatTooltipModule, TranslatePipe, FieldFileUpload, Image],
    templateUrl: "./field-image.html",
    styleUrl: "./field-image.scss",
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush,
    providers: [
        { provide: NG_VALUE_ACCESSOR, useExisting: forwardRef(() => FieldImage), multi: true },
        { provide: NG_VALIDATORS, useExisting: forwardRef(() => FieldImage), multi: true },
    ],
})
export class FieldImage implements OnChanges, ControlValueAccessor, Validator {
    @Input() // ".doc,.docx,.xls,.xlsx"; ".bmp,.gif"; "image/png,image/jpeg"; "audio/*,video/*,image/*";
    public accepts: string | null | undefined; // Define the file types (separated by commas) available for upload.
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
    public kind: string | null | undefined = IMAGE;
    @Input()
    public label: string | null | undefined; // "field-image-and-upload.label"
    @Input()
    public maxSize = -1;

    @Output()
    readonly addFile: EventEmitter<File> = new EventEmitter();
    @Output()
    readonly readFile: EventEmitter<string[]> = new EventEmitter();
    @Output()
    readonly deleteFile: EventEmitter<void> = new EventEmitter();

    private changeDetector: ChangeDetectorRef = inject(ChangeDetectorRef);

    public formControl: FormControl = new FormControl({ value: null, disabled: false }, []);
    public formGroup: FormGroup = new FormGroup({ image: this.formControl });

    public imageView: string = "";
    public initIsImage: boolean | undefined;
    public imageFile: File | null | undefined;

    ngOnChanges(changes: SimpleChanges): void {
        if (!!changes["isRequired"]) {
            this.prepareFormGroup(this.isRequired || null);
        }
        if (!!changes["isDisabled"]) {
            this.setDisabledState(!!this.isDisabled);
        }
        if (!!changes["errorMsg"]) {
            this.formControl.updateValueAndValidity();
            this.onChange(this.formControl.value);
        }
    }

    // ** ControlValueAccessor - start **

    public onChange: (val: string | null) => void = () => { };
    public onTouched: () => void = () => { };

    public writeValue(value: any): void {
        this.imageView = value || "";
        this.formControl.setValue(value, { emitEvent: true });
        if (this.initIsImage === undefined) {
            this.initIsImage = !!value;
        }
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

    public getErrorMsg(errors: ValidationErrors | null): string {
        const key = Object.keys(errors || {})[0];
        return !!key ? `417.${!!this.kind ? this.kind + ":" : ""}${key}` : "";
    }
    public getErrorObj(errors: ValidationErrors | null, value: string | null | undefined): ValidationErrors {
        const resObj = !!this.kind ? { [`${this.kind || ""}`]: value } : {};
        return { ...errors, ...resObj };
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

    public addImage(file: File): void {
        if (!!this.isDisabled || !!this.isReadOnly) {
            return;
        }
        this.imageFile = file;
        this.formControl.setValue(file.name, { emitEvent: true });
        this.onTouched();
        this.formControl.markAsTouched();
        this.onChange(file.name);
        this.addFile.emit(file);
    }

    public readImage(buffFile: string[]): void {
        if (!!this.isDisabled || !!this.isReadOnly) {
            return;
        }
        if (buffFile.length > 0) {
            this.imageView = buffFile[1];
            this.changeDetector.markForCheck();
        }
        this.readFile.emit(buffFile);
    }

    public deleteImage(): void {
        if (!!this.isDisabled || !!this.isReadOnly) {
            return;
        }
        this.imageFile = (!!this.initIsImage ? null : undefined);
        this.imageView = "";
        this.formControl.setValue(null, { emitEvent: true });
        this.onTouched();
        this.formControl.markAsTouched();
        this.onChange(null);
        this.deleteFile.emit();
    }

    // ** Private API **

    private errorMsgValidator = (control: AbstractControl): ValidationErrors | null => {
        return !!control && !!this.errorMsg ? { [CUSTOM_ERROR]: true } : null;
    };

    private prepareFormGroup(isRequired: boolean | null): void {
        this.formControl.clearValidators();
        const paramsObj = {
            ...(!!isRequired ? { "required": true } : {}),
        };
        this.formControl.setValidators([...ValidatorUtils.prepare(paramsObj), this.errorMsgValidator]);
        this.formControl.updateValueAndValidity();
    }

}
