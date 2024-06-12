export class Debounce {
  private timer: number | undefined;
  public run(callback: () => void, delay?: number | undefined): void {
    this.clear();
    this.timer = window.setTimeout(() => {
      this.clear();
      callback();
    }, delay);
  }
  public clear(): void {
    window.clearTimeout(this.timer);
    this.timer = undefined;
  }
}

/*
  const handlerChange = debounceFn((event) => handlerInput(event), 200);
  function handlerInput(event) { console.log(event.target.value); }
  <input (input)="handlerChange($event)" />
*/
export function debounceFn(this: any, func: (...arg0: any[]) => void, timeout = 300): (...args: any[]) => void {
  const context = this;
  let timer: number | undefined;
  return (...args: any[]) => {
    window.clearTimeout(timer);
    timer = window.setTimeout(() => {
      timer = undefined;
      func.apply(context, args);
    }, timeout);
  };
}

