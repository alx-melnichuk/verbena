import { ComponentFixture, TestBed } from '@angular/core/testing';

import { FieldLocaleComponent } from './field-locale.component';

describe('FieldLocaleComponent', () => {
  let component: FieldLocaleComponent;
  let fixture: ComponentFixture<FieldLocaleComponent>;

  beforeEach(() => {
    TestBed.configureTestingModule({
      imports: [FieldLocaleComponent]
    });
    fixture = TestBed.createComponent(FieldLocaleComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
