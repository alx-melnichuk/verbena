import {
    ChangeDetectionStrategy, ChangeDetectorRef, Component, EventEmitter, HostBinding, Input, OnChanges, Output, SimpleChanges,
    ViewEncapsulation, forwardRef
} from '@angular/core';
import { CommonModule } from '@angular/common';
import {
    AbstractControl, ControlValueAccessor, FormControl, FormGroup, NG_VALIDATORS, NG_VALUE_ACCESSOR, ReactiveFormsModule, ValidationErrors,
    Validator, ValidatorFn
} from '@angular/forms';
import { MatButtonModule } from '@angular/material/button';
import { MatInputModule } from '@angular/material/input';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatTooltipModule } from '@angular/material/tooltip';
import { TranslatePipe } from '@ngx-translate/core';
import { ValidatorUtils } from 'src/app/utils/validator.utils';

import { FieldFileUploadComponent } from '../field-file-upload/field-file-upload.component';

export const IMAGE_AND_UPLOAD = "image_and_upload";
export const CUSTOM_ERROR = 'customError';

/*
  <!-- ** 1 ** - The width and height dimensions correspond to the actual dimensions of the image. -->

  <div variant="1">
    <app-field-image-and-upload></app-field-image-and-upload>
  </div>

  <!-- ** 2 ** - The height of the element is specified. The aspect ratio of the image is preserved. -->

  <div variant="2a" style="height: 200px;">
    <app-field-image-and-upload hg100></app-field-image-and-upload>
  </div>
    
  <div variant="2b">
    <app-field-image-and-upload style="height: 200px;"></app-field-image-and-upload>
  </div>

  <div variant="2c">
    <app-field-image-and-upload style="--fiau-hg: 200px;"></app-field-image-and-upload>
  </div>

  <!-- ** 3 ** - The width of the element is specified. The aspect ratio of the image is preserved. -->

  <div variant="3a" style="width: 300px;">
    <app-field-image-and-upload wd100></app-field-image-and-upload>
  </div>
      
  <div variant="3b">
    <app-field-image-and-upload style="width: 300px; ---fiau-rmv-dl: -6px;"></app-field-image-and-upload>
  </div>
  
  <div variant="3c">
    <app-field-image-and-upload style="--fiau-wd: 300px; ---fiau-rmv-dl: -6px;"></app-field-image-and-upload>
  </div>

  <!-- ** 4 ** - The width and height of the element are specified. The aspect ratio of the image is preserved. -->

  <div variant="4a" style="width: 300px; height: 200px;">
    <app-field-image-and-upload wd100 hg100></app-field-image-and-upload>
  </div>
          
  <div variant="4b">
    <app-field-image-and-upload style="width: 300px; height: 200px; ---fiau-rmv-dl: -6px; --fiau-wd: 100%;">
    </app-field-image-and-upload>
  </div>
      
  <div variant="4c">
    <app-field-image-and-upload style="--fiau-wd: 300px; --fiau-hg: 200px; ---fiau-rmv-dl: -6px;">
    </app-field-image-and-upload>
  </div>
*/

@Component({
    selector: 'app-field-image-and-upload',
    standalone: true,
    exportAs: 'appFieldImageAndUpload',
    imports: [CommonModule, ReactiveFormsModule, MatButtonModule, MatInputModule, MatFormFieldModule, MatTooltipModule, TranslatePipe,
        FieldFileUploadComponent],
    templateUrl: './field-image-and-upload.component.html',
    styleUrl: './field-image-and-upload.component.scss',
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush,
    providers: [
        { provide: NG_VALUE_ACCESSOR, useExisting: forwardRef(() => FieldImageAndUploadComponent), multi: true },
        { provide: NG_VALIDATORS, useExisting: forwardRef(() => FieldImageAndUploadComponent), multi: true },
    ],
})
export class FieldImageAndUploadComponent implements OnChanges, ControlValueAccessor, Validator {
    @Input()
    public gist: string = IMAGE_AND_UPLOAD;
    @Input()
    // ".doc,.docx,.xls,.xlsx"; ".bmp,.gif"; "image/png,image/jpeg"; "audio/*,video/*,image/*";
    public accepts: string | null | undefined; // Define the file types (separated by commas) available for upload.
    @Input()
    public errorMsg: string | null | undefined;
    @Input()
    public hint: string = '';
    @Input()
    public isDisabled: boolean = false;
    @Input()
    public isReadonly: boolean = false;
    @Input()
    public isRequired: boolean = false;
    @Input()
    public label: string = 'field-image-and-upload.label';
    @Input()
    public maxSize = -1;

    @Output()
    readonly addFile: EventEmitter<File> = new EventEmitter();
    @Output()
    readonly readFile: EventEmitter<string[]> = new EventEmitter();
    @Output()
    readonly deleteFile: EventEmitter<void> = new EventEmitter();

    @HostBinding('class.is-disabled')
    public isDisabledVal: boolean = false;
    @HostBinding('class.is-non-event')
    public get isNonEvent(): boolean {
        return this.isDisabledVal || this.isReadonly;
    }

    public imageFile: File | null | undefined;
    public imageView: string = '';
    public initIsImage: boolean | undefined;

    public formControl: FormControl = new FormControl({ value: null, disabled: false }, []);
    public formGroup: FormGroup = new FormGroup({ image: this.formControl });

    constructor(
        private changeDetector: ChangeDetectorRef,
    ) {
    }

    ngOnChanges(changes: SimpleChanges): void {
        if (!!changes['isRequired']) {
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

    public onChange: (val: string | null) => void = () => { };
    public onTouched: () => void = () => { };

    public writeValue(value: any): void {
        this.imageView = value || '';
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
        this.isDisabledVal = isDisabled;
        isDisabled ? this.formGroup.disable() : this.formGroup.enable();
    }

    // ** ControlValueAccessor - finish **

    // ** Validator - start **

    public validate(control: AbstractControl): ValidationErrors | null {
        return this.formControl.errors;
    }

    // ** Validator - finish **

    // ** Public API **

    public getErrorMsg(errors: ValidationErrors | null): string {
        return ValidatorUtils.getErrorMsg(errors, this.gist || IMAGE_AND_UPLOAD);
    }

    public getFormControl(): FormControl {
        return this.formControl;
    }

    public addImage(file: File): void {
        if (this.isDisabledVal || this.isReadonly) {
            return;
        }
        this.imageFile = file;
        this.formControl.setValue(file.name, { emitEvent: true });
        this.onTouched();
        this.onChange(file.name);
        this.addFile.emit(file);
    }

    public readImage(buffFile: string[]): void {
        if (this.isDisabledVal || this.isReadonly) {
            return;
        }
        if (buffFile.length > 0) {
            this.imageView = buffFile[1];
            this.changeDetector.markForCheck();
        }
        this.readFile.emit(buffFile);
    }

    public deleteImage(): void {
        if (this.isDisabledVal || this.isReadonly) {
            return;
        }
        this.imageFile = (!!this.initIsImage ? null : undefined);
        this.imageView = '';
        this.formControl.setValue(null, { emitEvent: true });
        this.onTouched();
        this.onChange(null);
        this.deleteFile.emit();
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
        };
        this.formControl.setValidators([...ValidatorUtils.prepare(paramsObj), this.errorMsgValidator]);
        this.formControl.updateValueAndValidity();
    }
}
