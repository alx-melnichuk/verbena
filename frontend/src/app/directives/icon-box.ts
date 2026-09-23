import { Directive, ElementRef, inject, Input, OnChanges, Renderer2, SimpleChanges } from "@angular/core";

const ATTRIB_NAMES = ["viewBox", "fill", "stroke"];

@Directive({
    selector: "svg[appIconBox]",
    exportAs: "appIconBox",
    standalone: true,
})
export class IconBox implements OnChanges {
    private readonly element = inject(ElementRef<SVGElement>);
    private readonly renderer = inject(Renderer2);

    @Input()
    public mode: string | null | undefined;

    ngOnChanges(changes: SimpleChanges): void {
        if (!!changes["mode"]) {
            this.clear(this.renderer, this.element);
            switch (this.mode) {
                case "PasswdShow": this.createPasswdShow(this.renderer, this.element); break;
                case "PasswdHide": this.createPasswdHide(this.renderer, this.element); break;
                case "Diskette": this.createDiskette(this.renderer, this.element); break;
                case "DeleteDocument": this.createDeleteDocument(this.renderer, this.element); break;
                default:
                    break;
            }
        }
    }
    // ** Public API **

    // ** Private API **

    private clear(renderer: Renderer2, element: ElementRef<any>): void {
        const children = element.nativeElement.children;
        const elemList = Array.from(children);
        for (let idx = 0; idx < elemList.length; idx++) {
            renderer.removeChild(element.nativeElement, elemList[idx]);
        }

        const elem = element.nativeElement;
        const attrNames = Array.from(elem.attributes).map((attr: any) => attr.name);
        for (let idx = 0; idx < attrNames.length; idx++) {
            const attrName = attrNames[idx];
            if (ATTRIB_NAMES.indexOf(attrName.split("-")[0]) > -1) {
                renderer.removeAttribute(element.nativeElement, attrName);
            }
        }
    }
    private createPasswdShow(renderer: Renderer2, element: ElementRef<any>): void {
        const svg = element.nativeElement;

        renderer.setAttribute(svg, "viewBox", "0 0 24 24");
        renderer.setAttribute(svg, "fill", "currentColor");

        const path1 = renderer.createElement("path", "svg");
        renderer.setAttribute(path1, "d", "M23.911 11.715 C23.722 11.441 19.180 5 12.000 5 C5.840 5 0.350 11.404 0.119 11.677 C-0.0395 11.863 -0.040 12.136 0.119 12.323 C0.346 12.596 5.839 18.999 12.000 18.999 C18.161 18.999 23.651 12.596 23.882 12.322 C24.027 12.150 24.040 11.901 23.911 11.715 Z M12.000 15.999 C9.794 15.999 8.000 14.206 8.000 11.999 C8.000 9.794 9.794 7.999 12.000 7.999 C14.206 7.999 16.000 9.794 16.000 11.999 C16.000 14.206 14.206 15.999 12.000 15.999 Z");
        renderer.appendChild(svg, path1);
    }
    private createPasswdHide(renderer: Renderer2, element: ElementRef<any>): void {
        const svg = element.nativeElement;

        renderer.setAttribute(svg, "viewBox", "0 0 24 24");
        renderer.setAttribute(svg, "fill", "currentColor");

        const path1 = renderer.createElement("path", "svg");
        renderer.setAttribute(path1, "d", "m12.000,5 c-6.161,0 -11.651,6.404 -11.882,6.677 c-0.158,0.186 -0.158,0.459 0,0.646 c0.138,0.163 2.156,2.506 5.066,4.371 l3.134,-3.134 c-0.204,-0.48 -0.318,-1.007 -0.318,-1.561 c0,-2.206 1.795,-3.999 3.999,-3.999 c0.554,0 1.081,0.114 1.561,0.318 l2.577,-2.577 c-1.256,-0.449 -2.635,-0.7409 -4.138,-0.741 z");
        renderer.appendChild(svg, path1);

        const path2 = renderer.createElement("path", "svg");
        renderer.setAttribute(path2, "d", "m23.911,11.715 c-0.128,-0.185 -2.251,-3.173 -5.820,-5.099 l2.763,-2.763 c0.195,-0.195 0.195,-0.512 0,-0.707 c-0.195,-0.195 -0.512,-0.195 -0.707,0 l-16.999,16.999 c-0.195,0.195 -0.195,0.5121 0,0.707 c0.097,0.098 0.225,0.147 0.353,0.147 c0.128,0 0.256,-0.049 0.354,-0.146 l3.135,-3.135 c1.545,0.754 3.249,1.281 5.011,1.281 c6.161,0 11.651,-6.404 11.882,-6.677 c0.145,-0.173 0.158,-0.422 0.029,-0.608 z m-11.911,4.285 c-0.923,0 -1.762,-0.327 -2.440,-0.853 l5.587,-5.587 c0.526,0.678 0.853,1.517 0.853,2.440 c0,2.206 -1.794,3.999 -3.999,3.999 z");
        renderer.appendChild(svg, path2);
    }
    private createDiskette(renderer: Renderer2, element: ElementRef<any>): void {
        const svg = element.nativeElement;

        renderer.setAttribute(svg, "viewBox", "0 0 96 96");
        renderer.setAttribute(svg, "fill", "none");
        renderer.setAttribute(svg, "stroke", "currentColor");
        renderer.setAttribute(svg, "stroke-linecap", "round");
        renderer.setAttribute(svg, "stroke-linejoin", "round");
        renderer.setAttribute(svg, "stroke-width", "6");

        const path1 = renderer.createElement("path", "svg");
        renderer.setAttribute(path1, "d", "M13.2,3 h63.2 l16,15 v64.5 a10,10 0,0,1 -10,10 h-69.1 a10,10 0,0,1 -10,-10 v-69.5 a10,10 0,0,1 10,-10 z");
        renderer.appendChild(svg, path1);

        const path2 = renderer.createElement("path", "svg");
        renderer.setAttribute(path2, "d", "M21,3 v33 h46 v-33");
        renderer.appendChild(svg, path2);

        const path3 = renderer.createElement("path", "svg");
        renderer.setAttribute(path3, "fill", "currentColor");
        renderer.setAttribute(path3, "d", "M46.2,3 v23 h10 v-23 h-12 z");
        renderer.appendChild(svg, path3);

        const path4 = renderer.createElement("path", "svg");
        renderer.setAttribute(path4, "d", "M16,92 v-38 h65 v38");
        renderer.appendChild(svg, path4);
    }
    private createDeleteDocument(renderer: Renderer2, element: ElementRef<any>): void {
        const svg = element.nativeElement;

        renderer.setAttribute(svg, "viewBox", "0 0 96 96");
        renderer.setAttribute(svg, "fill", "none");
        renderer.setAttribute(svg, "stroke", "currentColor");
        renderer.setAttribute(svg, "stroke-linecap", "round");
        renderer.setAttribute(svg, "stroke-linejoin", "round");
        renderer.setAttribute(svg, "stroke-width", "6");

        const path1 = renderer.createElement("path", "svg");
        renderer.setAttribute(path1, "d", "M64,12.0  h-28 a4,4 0,0,0 -4,2  l-14,14 a4,4 0,0,0 -2,4  v44 a8,8 0,0,0 8,8 h12 m36,-48 v-16 a8,8 0,0,0 -8,-8  m-48,20 h14 a6,6 0,0,0 6,-6 v-14");
        renderer.appendChild(svg, path1);

        const path2 = renderer.createElement("path", "svg");
        renderer.setAttribute(path2, "d", "M80,55 a22,22 0,1,1 -40,0 22,22 0,0,1 40,0 Z");
        renderer.appendChild(svg, path2);

        const path3 = renderer.createElement("path", "svg");
        renderer.setAttribute(path3, "d", "M52,56 l16,16 m0,-16 l-16,16");
        renderer.appendChild(svg, path3);
    }
}
