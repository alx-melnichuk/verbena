import { ComponentFixture, TestBed } from '@angular/core/testing';

import { FieldThemeComponent } from './field-theme.component';

describe('FieldThemeComponent', () => {
  let component: FieldThemeComponent;
  let fixture: ComponentFixture<FieldThemeComponent>;

  beforeEach(() => {
    TestBed.configureTestingModule({
      imports: [FieldThemeComponent]
    });
    fixture = TestBed.createComponent(FieldThemeComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
