import { Directive, EventEmitter, HostBinding, HostListener, Output } from '@angular/core';

@Directive({
    selector: '[appFieldDragAndDrop]',
    standalone: true
})
export class FieldDragAndDropDirective {

    @Output()
    readonly fileDropped: EventEmitter<FileList> = new EventEmitter();

    constructor() {
    }

    @HostBinding('class.fileover')
    public fileOver = false;

    // A dragged element is over the drop target.
    @HostListener('dragover', ['$event'])
    public onDragOver(evt: DragEvent): void {
        evt.preventDefault();
        evt.stopPropagation();
        this.fileOver = true;
    }

    // A dragged element leaves the drop target.
    @HostListener('dragleave', ['$event'])
    public onDragLeave(evt: DragEvent): void {
        evt.preventDefault();
        evt.stopPropagation();
        this.fileOver = false;
    }

    // A dragged element is dropped on the target.
    @HostListener('drop', ['$event'])
    public onDrop(evt: DragEvent): void {
        evt.preventDefault();
        evt.stopPropagation();
        this.fileOver = false;
        const files = evt.dataTransfer?.files;
        if (!!files && files.length > 0) {
            this.fileDropped.emit(files);
        }
    }
}
