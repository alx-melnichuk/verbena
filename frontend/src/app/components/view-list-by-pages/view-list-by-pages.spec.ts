import { ComponentFixture, TestBed } from '@angular/core/testing';

import { ViewListByPages } from './view-list-by-pages';

describe('ViewListByPages', () => {
  let component: ViewListByPages;
  let fixture: ComponentFixture<ViewListByPages>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [ViewListByPages]
    })
    .compileComponents();

    fixture = TestBed.createComponent(ViewListByPages);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
